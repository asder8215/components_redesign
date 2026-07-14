#![feature(path_trailing_sep)]
#![allow(dead_code)]
#![allow(unused)]
use core::slice;
use std::{
    cmp, ffi::OsStr, fmt, hash::{Hash, Hasher}, hint::black_box, iter::FusedIterator, marker::PhantomData, ops::Index, os::unix::ffi::OsStrExt, path::{MAIN_SEPARATOR, Path, PathBuf}
};

macro_rules! path_separator_bytes {
    ($($sep:literal),+) => (
        pub const SEPARATORS: &[char] = &[$($sep as char,)+];
        pub const SEPARATORS_STR: &[&str] = &[$(
            match str::from_utf8(&[$sep]) {
                Ok(s) => s,
                Err(_) => panic!("path_separator_bytes must be ASCII bytes"),
            }
        ),+];

        #[inline]
        pub const fn is_sep_byte(b: u8) -> bool {
            $(b == $sep) ||+
        }
    )
}

path_separator_bytes!(b'/');
pub const MAIN_SEPARATOR_STR: &str = SEPARATORS_STR[0];

pub const HAS_PREFIXES: bool = false;

unsafe fn from_u8_slice(s: &[u8]) -> &Path {
    unsafe { Path::new(OsStr::from_encoded_bytes_unchecked(s)) }
}

fn parse_prefix(_: &OsStr) -> Option<Prefix<'_>> {
    None
}

#[derive(Copy, Clone, Debug, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Prefix<'a> {
    /// Verbatim prefix, e.g., `\\?\cat_pics`.
    ///
    /// Verbatim prefixes consist of `\\?\` immediately followed by the given
    /// component.
    Verbatim(&'a OsStr),

    /// Verbatim prefix using Windows' _**U**niform **N**aming **C**onvention_,
    /// e.g., `\\?\UNC\server\share`.
    ///
    /// Verbatim UNC prefixes consist of `\\?\UNC\` immediately followed by the
    /// server's hostname and a share name.
    VerbatimUNC(&'a OsStr, &'a OsStr),

    /// Verbatim disk prefix, e.g., `\\?\C:`.
    ///
    /// Verbatim disk prefixes consist of `\\?\` immediately followed by the
    /// drive letter and `:`.
    VerbatimDisk(u8),

    /// Device namespace prefix, e.g., `\\.\COM42`.
    ///
    /// Device namespace prefixes consist of `\\.\` (possibly using `/`
    /// instead of `\`), immediately followed by the device name.
    DeviceNS(&'a OsStr),

    /// Prefix using Windows' _**U**niform **N**aming **C**onvention_, e.g.
    /// `\\server\share`.
    ///
    /// UNC prefixes consist of the server's hostname and a share name.
    UNC(&'a OsStr, &'a OsStr),

    /// Prefix `C:` for the given disk drive.
    Disk(u8),
}

impl<'a> Prefix<'a> {
    #[inline]
    fn len(&self) -> usize {
        use self::Prefix::*;
        fn os_str_len(s: &OsStr) -> usize {
            s.as_encoded_bytes().len()
        }
        match *self {
            Verbatim(x) => 4 + os_str_len(x),
            VerbatimUNC(x, y) => {
                8 + os_str_len(x)
                    + if os_str_len(y) > 0 {
                        1 + os_str_len(y)
                    } else {
                        0
                    }
            }
            VerbatimDisk(_) => 6,
            UNC(x, y) => {
                2 + os_str_len(x)
                    + if os_str_len(y) > 0 {
                        1 + os_str_len(y)
                    } else {
                        0
                    }
            }
            DeviceNS(x) => 4 + os_str_len(x),
            Disk(_) => 2,
        }
    }

    #[inline]
    pub fn is_verbatim(&self) -> bool {
        use self::Prefix::*;
        matches!(*self, Verbatim(_) | VerbatimDisk(_) | VerbatimUNC(..))
    }

    #[inline]
    fn is_drive(&self) -> bool {
        matches!(*self, Prefix::Disk(_))
    }

    #[inline]
    fn has_implicit_root(&self) -> bool {
        !self.is_drive()
    }
}

#[derive(Copy, Clone, Eq, Debug)]
pub struct PrefixComponent<'a> {
    /// The prefix as an unparsed `OsStr` slice.
    raw: &'a OsStr,

    /// The parsed prefix data.
    parsed: Prefix<'a>,
}

impl<'a> PrefixComponent<'a> {
    /// Returns the parsed prefix data.
    ///
    /// See [`Prefix`]'s documentation for more information on the different
    /// kinds of prefixes.
    #[must_use]
    #[inline]
    pub fn kind(&self) -> Prefix<'a> {
        self.parsed
    }

    /// Returns the raw [`OsStr`] slice for this prefix.
    #[must_use]
    #[inline]
    pub fn as_os_str(&self) -> &'a OsStr {
        self.raw
    }
}

impl<'a> PartialEq for PrefixComponent<'a> {
    #[inline]
    fn eq(&self, other: &PrefixComponent<'a>) -> bool {
        self.parsed == other.parsed
    }
}

impl<'a> PartialOrd for PrefixComponent<'a> {
    #[inline]
    fn partial_cmp(&self, other: &PrefixComponent<'a>) -> Option<cmp::Ordering> {
        PartialOrd::partial_cmp(&self.parsed, &other.parsed)
    }
}

impl Ord for PrefixComponent<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        Ord::cmp(&self.parsed, &other.parsed)
    }
}

impl Hash for PrefixComponent<'_> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.parsed.hash(h);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Component<'a> {
    /// A Windows path prefix, e.g., `C:` or `\\server\share`.
    ///
    /// There is a large variety of prefix types, see [`Prefix`]'s documentation
    /// for more.
    ///
    /// Does not occur on Unix.
    Prefix(PrefixComponent<'a>),

    /// The root directory component, appears after any prefix and before anything else.
    ///
    /// It represents a separator that designates that a path starts from root.
    RootDir,

    /// A reference to the current directory, i.e., `.`.
    CurDir,

    /// A reference to the parent directory, i.e., `..`.
    ParentDir,

    /// A normal component, e.g., `a` and `b` in `a/b`.
    ///
    /// This variant is the most common one, it represents references to files
    /// or directories.
    Normal(&'a OsStr),
}

impl<'a> Component<'a> {
    /// Extracts the underlying [`OsStr`] slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    ///
    /// let path = Path::new("./tmp/foo/bar.txt");
    /// let components: Vec<_> = path.components().map(|comp| comp.as_os_str()).collect();
    /// assert_eq!(&components, &[".", "tmp", "foo", "bar.txt"]);
    /// ```
    #[must_use = "`self` will be dropped if the result is not used"]
    pub fn as_os_str(self) -> &'a OsStr {
        match self {
            Component::Prefix(p) => p.as_os_str(),
            Component::RootDir => OsStr::new(MAIN_SEPARATOR_STR),
            Component::CurDir => OsStr::new("."),
            Component::ParentDir => OsStr::new(".."),
            Component::Normal(path) => path,
        }
    }
}

/// This is what the first component of our path is
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum FirstComponent {
    /// For all paths starting with `/`
    AbsolutePath,
    /// For paths without root path like `.`, `..`, `a/`
    // RelativePath,
    /// For Window specific paths like (`C:`, `\\?\UNC\server\share`,
    /// `\\.\COM42`, etc.)
    Prefix,
}

#[derive(Clone)]
pub struct Components<'a> {
    path: &'a [u8],
    has_physical_root: bool,
    is_done: bool,
    // prefix: Option<PrefixComponent<'a>>,
    // I wouldn't need this field if I had `Prefix`
    first_comp: Option<FirstComponent>,
}

impl<'a> Components<'a> {
    /// Is the *original* path rooted?
    #[inline]
    fn has_root(&self) -> bool {
        self.has_physical_root /* ||  self.prefix.map(|prefix| prefix.parsed.has_implicit_root()).unwrap_or(false) */
    }

    // #[inline]
    // fn prefix_verbatim(&self) -> bool {
    //     if !HAS_PREFIXES {
    //         return false;
    //     }
    //     self.prefix.as_ref().map(Prefix::is_verbatim).unwrap_or(false)
    // }

    /// Normalizes away trailing separators and current directory ('.') components
    /// in the forward direction. Returns the 0-index `self.path` should start at
    /// to subslice at in the front direction.
    #[inline]
    fn normalize_front(&mut self, mut front: usize) -> usize {
        let mut cur_dir_present = false;
        match self.path[front..].iter().position(|b| {
            if !is_sep_byte(*b) {
                if *b == b'.' && !cur_dir_present {
                    cur_dir_present = true;
                    false
                } else {
                    true
                }
            } else {
                cur_dir_present = false;
                false
            }
        }) {
            None => {
                self.is_done = true;
                return self.path.len()
            },
            Some(i) => {
                if cur_dir_present {
                    front += i - 1;
                } else {
                    front += i;
                }
            }
        }
        front
    }

    /// Normalizes away trailing separators and current directory ('.') components
    /// in the backward direction. Returns the 1-index `self.path` should start at
    /// to find next separator in the back direction.
    #[inline]
    fn normalize_back(&mut self) -> usize {
        let mut cur_dir_present = false;
        match self.path.iter().rposition(|b| {
            if !is_sep_byte(*b) {
                if *b == b'.' && !cur_dir_present {
                    cur_dir_present = true;
                    false
                } else {
                    true
                }
            } else {
                cur_dir_present = false;
                false
            }
        }) {
            None => {
                // For cases like "./a", where our path
                // will observe "." at the end, and we need to return
                // that we observed "." component instead of
                // returning an empty path.
                if cur_dir_present {
                    return 1;
                } else {
                    self.is_done = true;
                    return 0;
                }
            }
            Some(i) => {
                if cur_dir_present {
                    return i + 2;
                } else {
                    return i + 1;
                }
            }
        }
    }

    /// Parse a u8 slice into an OsStr, which is encoded into a `Component`
    #[inline]
    fn parse_single_component(&self, slice: &'a [u8]) -> Option<Component<'a>> {
        match slice {
            [] => return None,
            [b'.'] => Some(Component::CurDir),
            [b'.', b'.'] => Some(Component::ParentDir),
            _ => {
                let root_slice = [MAIN_SEPARATOR as u8];
                if slice == root_slice {
                    return Some(Component::RootDir);
                }
                // SAFETY: Our sliced path is guaranteed to capture the entire component
                // due to delimiting on ascii separators from front and back.
                let path_osstr = unsafe { OsStr::from_encoded_bytes_unchecked(slice) };
                Some(Component::Normal(path_osstr))
            }
        }
    }

    /// Extracts a slice corresponding to the portion of the path remaining for iteration.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    ///
    /// let mut components = Path::new("/tmp/foo/bar.txt").components();
    /// components.next();
    /// components.next();
    ///
    /// assert_eq!(Path::new("foo/bar.txt"), components.as_path());
    /// ```
    #[must_use]
    pub fn as_path(&self) -> &'a Path {
        let mut cur_dir_present = false;
        let (done, back) = match self.path.iter().rposition(|b| {
            if !is_sep_byte(*b) {
                if *b == b'.' && !cur_dir_present {
                    cur_dir_present = true;
                    false
                } else {
                    true
                }
            } else {
                cur_dir_present = false;
                false
            }
        }) {
            None => {
                // For cases like "./a", where our path
                // will observe "." at the end, and we need to return
                // that we observed "." component instead of
                // returning an empty path.
                if cur_dir_present {
                    (false, 1)
                } else {
                    // self.is_done = true;
                    (true, 0)
                }
            }
            Some(i) => {
                if cur_dir_present {
                    (false, i + 2)
                } else {
                    (false, i + 1)
                }
            }
        };

        if done && self.has_root() {
            return Path::new("/");
        }

        // SAFETY: self.path contains a valid Path. What `end` stores is 
        // the 1-indexing of the last byte we should normalize away, so 
        // we should have a valid slice subslicing from there.
        unsafe { from_u8_slice(&self.path[..back]) }
    }

    /// Parses the next component in `Components<'_>` from the left
    #[inline]
    fn parse_next_component(&mut self) -> (usize, Option<Component<'a>>) {
        // Finds the next separator in the back direction
        let (ind, comp) = match self.path.iter().position(|b| is_sep_byte(*b)) {
            None => {
                // self.is_done = true;
                (self.path.len(), self.path)
            },
            Some(i) => (i, &self.path[..i]),
        };
        
        (ind, self.parse_single_component(comp))
    }

    /// Parses the next back component in `Components<'_>` from the
    /// right
    #[inline]
    fn parse_next_back_component(&mut self, mut back: usize) -> (usize, Option<Component<'a>>) {
        // Finds the next separator in the front direction
        let (size, comp) = match self.path[..back].iter().rposition(|b| is_sep_byte(*b)) {
            None => {
                self.is_done = true;
                (0, &self.path[0..back])
            },
            Some(i) => (i, &self.path[i+1..back]),
        };

        (size, self.parse_single_component(comp))
    }
}

impl<'a> Iterator for Components<'a> {
    type Item = Component<'a>;

    #[inline]
    fn next(&mut self) -> Option<Component<'a>> {
        // We reach this case when we no longer have anymore paths
        // to consume (return `None`), or if our front idx was initially
        // equal to back idx (e.g. if we had `C:`, `.`, `/`)
        if !self.is_done {

            match self.first_comp {
                Some(FirstComponent::AbsolutePath) => {
                    self.has_physical_root = false;
                    let end_ind = self.normalize_front(0);
                    // let (size, comp) = self.parse_next_component();
                    self.path = if self.is_done {
                        // self.is_done = true;
                        &[]
                    } else {
                        &self.path[end_ind..]
                    };
                    self.first_comp = None;
                    return Some(Component::RootDir);
                },
                // Some(FirstComponent::Prefix) => {
                //     let prefix_comp = self.prefix.take().unwrap();

                //     if self.has_physical_root {
                //         self.first_comp = Some(FirstComponent::AbsolutePath);
                //     } else {
                //         self.first_comp = None;
                //     }

                //     return Some(Component::Prefix(prefix_comp));
                // },
                _ => {
                    let (size, comp) = self.parse_next_component();
                    let normalized_front_ind = self.normalize_front(size);

                    self.path = if self.is_done {
                        &[]
                    } else {
                        &self.path[normalized_front_ind..]
                    };

                    return comp;
                }
            }
            // if self.prefix.is_some() {
            //     let prefix = self.prefix.unwrap();
            //     self.prefix = None;
            //     let comp = &self.path[..prefix.len()];
            //     self.path = &self.path[prefix.len()..];

            //     // SAFETY: We already parsed the `Prefix` component when constructing
            //     // `Component` struct, which is stored in the prefix field. Slicing the 
            //     // path_bytes with length of the prefix field should then give us a valid
            //     // u8 slice representing the prefix. 
            //     return Some(Component::Prefix(PrefixComponent { raw: unsafe { OsStr::from_encoded_bytes_unchecked(comp) }, parsed: prefix }));
            // }

            // match self.first_comp {

            // }
            
            // if self.has_physical_root {
            //     self.has_physical_root = false;
            //     let end_ind = self.normalize_front(0);
            //     // let (size, comp) = self.parse_next_component();
            //     self.path = if self.is_done {
            //         // self.is_done = true;
            //         &[]
            //     } else {
            //         &self.path[end_ind..]
            //     };
            //     return Some(Component::RootDir);
            // }
            // let (size, comp) = self.parse_next_component();

            // let normalized_front_ind = self.normalize_front(size);

            // self.path = if self.is_done {
            //     &[]
            // } else {
            //     &self.path[normalized_front_ind..]
            // };

            // return comp;
        }

        // if let Some(prefix_comp) = self.prefix {
        //     self.prefix = None;
        //     self.first_comp = None;
        //     return Some(Component::Prefix(prefix_comp));
        // }

        None
    }
}

impl<'a> DoubleEndedIterator for Components<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Component<'a>> {
        // We reach here when we no longer have anymore paths
        // to consume, we're dealing with relative paths and
        // need to output "", or we need to output Prefix component
        // if !self.path.is_empty() {
        if !self.is_done {
            let back = self.normalize_back();
            if self.is_done && self.has_physical_root {
                self.has_physical_root = false;
                if matches!(self.first_comp, Some(FirstComponent::AbsolutePath)) {
                    self.first_comp = None;
                }
                self.path = &[];
                return Some(Component::RootDir);
            }
            let (size, comp) = self.parse_next_back_component(back);

            self.path = &self.path[..size];
            return comp;
        }

        // if let Some(prefix_comp) = self.prefix {
        //     self.prefix = None;
        //     self.first_comp = None;
        //     return Some(Component::Prefix(prefix_comp));
        // }

        None
    }
}

impl FusedIterator for Components<'_> {}

impl<'a> PartialEq for Components<'a> {
    #[inline]
    fn eq(&self, other: &Components<'a>) -> bool {
        // Fast path for exact matches, e.g. for hashmap lookups. 
        if self.path == other.path {
            // if self.prefix.is_none() && other.prefix.is_none(){
            //     return true;
            // } else if let Some(self_prefix) = self.prefix &&
            //     let Some(other_prefix) = other.prefix {
            //         return self_prefix.raw == other_prefix.raw;
            // }
            // else {
            //     return false;
            // }
            return true;
        }

        let mut left = self.clone();
        let mut right = other.clone();

        // if left.prefix.is_none() && right.prefix.is_none() {
        //     let first_difference = match left.path.iter().zip(right.path).rposition(|(&a, &b)| a != b) {
        //         None => left.path.len().min(right.path.len()),
        //         Some(diff) => left.path.len().min(right.path.len()) - diff - 1,
        //     };

        //     // let left_byte = left.path[first_difference];
        //     // let right_byte = 

        //     if let Some(previous_sep) =
        //         left.path[left.path.len() - first_difference..].iter().position(|&b| is_sep_byte(b))
        //     {
        //         // previous_sep;
        //         left.path = &left.path[previous_sep..];
        //         left.has_physical_root = false;
        //         left.first_comp = None;
        //         // right.path = &right.path[..previous_sep];
        //         // right.has_physical_root = false;
        //         // right.first_comp = None;
        //     }

        //     if let Some(previous_sep) = right.path[right.path.len() - first_difference..].iter().rposition(|&b| is_sep_byte(b))
        //     {
        //         right.path = &right.path[previous_sep..];
        //         right.has_physical_root = false;
        //         right.first_comp = None;
        //     }
        // }

        // if left.prefix.is_none() && right.prefix.is_none() {
        //     let first_difference = match left.path.iter().zip(right.path).position(|(&a, &b)| a != b) {
        //         None => left.path.len().min(right.path.len()),
        //         Some(diff) => diff,
        //     };

        //     if let Some(previous_sep) =
        //         left.path[..first_difference].iter().rposition(|&b| is_sep_byte(b))
        //     {
        //         let mismatched_component_start = previous_sep + 1;
        //         left.path = &left.path[left.normalize_front(mismatched_component_start)..];
        //         left.has_physical_root = false;
        //         left.first_comp = None;
        //         right.path = &right.path[right.normalize_front(mismatched_component_start)..];
        //         right.has_physical_root = false;
        //         right.first_comp = None;
        //     }
        // }

        // // compare back to front since absolute paths often share long prefixes
        // // Iterator::eq(self.clone().rev(), other.clone().rev())
        // Iterator::eq(left, right)

        // if left.prefix.is_none() && right.prefix.is_none() {
            let first_difference = match left.path.iter().zip(right.path).rposition(|(&a, &b)| a != b) {
                None => left.path.len().min(right.path.len()),
                Some(diff) => left.path.len().min(right.path.len()) - diff - 1,
            };

            // let left_byte = left.path[first_difference];
            // let right_byte = 

            if let Some(previous_sep) =
                left.path[left.path.len() - first_difference..].iter().position(|&b| is_sep_byte(b))
            {
                // previous_sep;
                left.path = &left.path[previous_sep..];
                left.has_physical_root = false;
                left.first_comp = None;
                // right.path = &right.path[..previous_sep];
                // right.has_physical_root = false;
                // right.first_comp = None;
            }

            if let Some(previous_sep) = right.path[right.path.len() - first_difference..].iter().rposition(|&b| is_sep_byte(b))
            {
                right.path = &right.path[previous_sep..];
                right.has_physical_root = false;
                right.first_comp = None;
            }
        // }

        Iterator::eq(left.rev(), right.rev())
    }
}

impl Eq for Components<'_> {}

impl<'a> PartialOrd for Components<'a> {
    #[inline]
    fn partial_cmp(&self, other: &Components<'a>) -> Option<cmp::Ordering> {
        Some(compare_components(self.clone(), other.clone()))
    }
}

impl Ord for Components<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        compare_components(self.clone(), other.clone())
    }
}

impl AsRef<Path> for Components<'_> {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AsRef<OsStr> for Components<'_> {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self.as_path().as_os_str()
    }
}

fn has_physical_root(s: &[u8], prefix: Option<Prefix<'_>>) -> bool {
    let path = if let Some(p) = prefix { &s[p.len()..] } else { s };
    !path.is_empty() && is_sep_byte(path[0])
}

fn eq_components(mut left: Components<'_>, mut right: Components<'_>) -> bool {
    // if left.prefix.is_none() && right.prefix.is_none() {
        // One of them is an empty path
        if left.is_done != left.is_done {
            return false;
        }

        // Both are empty paths
        if left.is_done && right.is_done {
            return true;
        }
    // }

    let mut left_iter = left.path.iter();
    let mut right_iter = right.path.iter();
    let mut bytes_consumed = 0;

    let (left_diff, right_diff) = 'diff: {
        while let Some(left_byte) = left_iter.next_back() && let Some(right_byte) = right_iter.next_back() {
            bytes_consumed += 1;
            if left_byte != right_byte {
                break 'diff (left.path.len() - bytes_consumed, right.path.len() - bytes_consumed)
            }
        }
        
        (left.path.len() - bytes_consumed, right.path.len() - bytes_consumed)
    };

    if let Some(next_sep) =
        left.path[left_diff..].iter().position(|&b| is_sep_byte(b))
    {
        left.path = &left.path[..left_diff + next_sep + 1];
    }

    if let Some(next_sep) =
        right.path[right_diff..].iter().position(|&b| is_sep_byte(b))
    {
        right.path = &right.path[..right_diff + next_sep + 1];
    }

    Iterator::eq(left.rev(), right.rev())
}

fn compare_components(mut left: Components<'_>, mut right: Components<'_>) -> cmp::Ordering {
    // Fast path for long shared prefixes
    //
    // - compare raw bytes to find first mismatch
    // - backtrack to find separator before mismatch to avoid ambiguous parsings of '.' or '..' characters
    // - if found update state to only do a component-wise comparison on the remainder,
    //   otherwise do it on the full path

    let first_difference = match left.path.iter().zip(right.path).position(|(&a, &b)| a != b) {
        None if left.path.len() == right.path.len() /*&& left.prefix.is_none() && right.prefix.is_none()*/ => return cmp::Ordering::Equal,
        None => left.path.len().min(right.path.len()),
        Some(diff) => diff,
    };

    if let Some(previous_sep) =
        left.path[..first_difference].iter().rposition(|&b| is_sep_byte(b))
    {
        let mismatched_component_start = previous_sep + 1;
        left.path = &left.path[left.normalize_front(mismatched_component_start)..];
        left.has_physical_root = false;
        left.first_comp = None;
        right.path = &right.path[right.normalize_front(mismatched_component_start)..];
        right.has_physical_root = false;
        right.first_comp = None;
    }

    Iterator::cmp(left, right)
}

fn components(path: &Path) -> Components<'_> {
    let os_str_path = path.as_os_str();
    // let prefix = parse_prefix(os_str_path);
    // let (prefix_comp, path_bytes) = if let Some(prefix) = prefix {
    //     let path = os_str_path.as_encoded_bytes();
    //     let prefix_len = prefix.len();
    //     if path.len() == prefix_len {
    //         (Some(PrefixComponent { raw: unsafe { OsStr::from_encoded_bytes_unchecked(path) }, parsed: prefix}), &path[..0])
    //     } else {
    //         (Some(PrefixComponent { raw: unsafe { OsStr::from_encoded_bytes_unchecked(&path[..prefix_len]) }, parsed: prefix}), &path[prefix_len..])
    //     }
    // } else {
    //     let path = os_str_path.as_encoded_bytes();
    //     (None, path)
    // };

    let path_bytes = os_str_path.as_encoded_bytes();

    // let has_physical_root = has_physical_root(path_bytes, prefix);
    let has_physical_root = has_physical_root(path_bytes, None);
    let first_comp = /*if prefix.is_some() {
        Some(FirstComponent::Prefix)
    } else */if has_physical_root {
        Some(FirstComponent::AbsolutePath)
    } else {
        None
    };

    let mut components = Components {
        path: path_bytes,
        has_physical_root,
        is_done: path_bytes.is_empty(),
        // prefix: prefix_comp,
        first_comp
    };

    components
}

#[inline]
fn _eq_comps(path: &Path, other: &Path) -> bool {
    path.as_os_str() == other.as_os_str() || eq_components(components(path), components(other))
}

#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Iter<'a> {
    inner: Components<'a>,
}

impl fmt::Debug for Iter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DebugHelper<'a>(&'a Path);

        impl fmt::Debug for DebugHelper<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.iter()).finish()
            }
        }

        f.debug_tuple("Iter")
            .field(&DebugHelper(self.as_path()))
            .finish()
    }
}

impl<'a> Iter<'a> {
    /// Extracts a slice corresponding to the portion of the path remaining for iteration.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    ///
    /// let mut iter = Path::new("/tmp/foo/bar.txt").iter();
    /// iter.next();
    /// iter.next();
    ///
    /// assert_eq!(Path::new("foo/bar.txt"), iter.as_path());
    /// ```
    #[must_use]
    #[inline]
    pub fn as_path(&self) -> &'a Path {
        self.inner.as_path()
    }
}

impl AsRef<Path> for Iter<'_> {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AsRef<OsStr> for Iter<'_> {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self.as_path().as_os_str()
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a OsStr;

    #[inline]
    fn next(&mut self) -> Option<&'a OsStr> {
        self.inner.next().map(Component::as_os_str)
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a OsStr> {
        self.inner.next_back().map(Component::as_os_str)
    }
}

impl FusedIterator for Iter<'_> {}

fn components_iter(path: &Path) {
    let comps = components(path);
    for comp in comps {}
}

fn components_next_iter(path: &Path) {
    let mut comps = Iter {
        inner: components(path),
    };
    while let Some(comp) = comps.next() {}
    // let mut comps = path.components();
    // while let Some(comp) = comps.next() {}
}

fn components_next_back_iter(path: &Path) {
    let mut comps = Iter {
        inner: components(path),
    };
    while let Some(comp) = comps.next_back() {}
    // let mut comps = path.components();
    // while let Some(comp) = comps.next_back() {}
}

fn path_iter(path: &Path) {
    let comps = Iter {
        inner: components(path),
    };
    for comp in comps {}
}

fn as_path_iter(path: &Path) {
    let mut comps = Iter {
        inner: components(path),
    };
    while let Some(comp) = comps.next() {
        let path = comps.as_path();
    }
}

fn eq_comps(path: &Path, other_path: &Path) {
    // let comp = components(path);
    // let other_comp = components(other_path);
    // comp == other_comp;
    // path > other_path;
    path.components() == other_path.components();
    // path == other_path;
    // _eq_comps(path, other_path);
}

fn compare_comps(path: &Path, other_path: &Path) {
    // let comp = components(path);
    // let other_comp = components(other_path);
    // path > other_path;
    let comp = path.components();
    let other_comp = other_path.components();
    // println!("{:?}", comp > other_comp);
    comp > other_comp;
}

fn main() {
    let mut path = String::from("/");
    let chars = vec!["a"; 108];
    let mut str = chars.join("");
    str.push('/');

    for i in 0..1000 {
        path.push_str(&str);
    }

    let path_b = format!("/b/{path}");
    // let path_b = format!("{path}/b/");

    for i in 0..10000 {
        // compare_comps(path.as_ref(), path_b.as_ref());
        eq_comps(path.as_ref(), path_b.as_ref());

        // components_next_iter(path.as_ref());
    }
}
