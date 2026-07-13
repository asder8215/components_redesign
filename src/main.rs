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
    RelativePath,
    /// For Window specific paths like (`C:`, `\\?\UNC\server\share`,
    /// `\\.\COM42`, etc.)
    Prefix,
}

#[derive(Clone)]
pub struct Components<'a> {
    path: &'a [u8],
    has_physical_root: bool,
    // prefix: Option<Prefix<'a>>,
    is_done: bool,
}

impl<'a> Components<'a> {
    /// Is the *original* path rooted?
    #[inline]
    fn has_root(&self) -> bool {
        // if !self.path.is_empty() {
        //     return is_sep_byte(self.path[0]);
        // }

        // false
        !self.path.is_empty() && is_sep_byte(self.path[0])
    }

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
            Some(i) => (i + 1, &self.path[..i]),
        };

        // let end_ind = self.normalize_front(ind);
        
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
            Some(i) => (i + 1, &self.path[i+1..back]),
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
            if self.has_physical_root {
                self.has_physical_root = false;
                let end_ind = self.normalize_front(0);
                // let (size, comp) = self.parse_next_component();
                self.path = if self.is_done {
                    // self.is_done = true;
                    &[]
                } else {
                    &self.path[end_ind..]
                };
                return Some(Component::RootDir);
            }
            let (size, comp) = self.parse_next_component();

            let normalized_front_ind = self.normalize_front(size);

            self.path = if self.is_done {
                &[]
            } else {
                &self.path[normalized_front_ind..]
            };

            return comp;
        }

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
                self.path = &[];
                return Some(Component::RootDir);
            }
            let (size, comp) = self.parse_next_back_component(back);

            self.path = &self.path[..size];
            return comp;
        }

        None
    }
}

impl FusedIterator for Components<'_> {}

impl<'a> PartialEq for Components<'a> {
    #[inline]
    fn eq(&self, other: &Components<'a>) -> bool {
        // Fast path for exact matches, e.g. for hashmap lookups.
        // Don't explicitly compare the prefix or has_physical_root fields since they'll
        // either be covered by the `path` buffer or are only relevant for `prefix_verbatim()`.
        if self.path.len()  == other.path.len()
        {
            if self.path == other.path {
                return true;
            }
        }

        // compare back to front since absolute paths often share long prefixes
        Iterator::eq(self.clone().rev(), other.clone().rev())
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

fn has_physical_root(s: &[u8]) -> bool {
    // let path = if let Some(p) = prefix {
    //     &s[p.len()..]
    // } else {
    //     s
    // };
    !s.is_empty() && is_sep_byte(s[0])
}

fn compare_components(mut left: Components<'_>, mut right: Components<'_>) -> cmp::Ordering {
    // Fast path for long shared prefixes
    //
    // - compare raw bytes to find first mismatch
    // - backtrack to find separator before mismatch to avoid ambiguous parsings of '.' or '..' characters
    // - if found update state to only do a component-wise comparison on the remainder,
    //   otherwise do it on the full path

    let first_difference = match left.path.iter().zip(right.path).position(|(&a, &b)| a != b) {
        None if left.path.len() == right.path.len() => return cmp::Ordering::Equal,
        None => left.path.len().min(right.path.len()),
        Some(diff) => diff,
    };

    if let Some(previous_sep) =
        left.path[..first_difference].iter().rposition(|&b| is_sep_byte(b))
    {
        let mismatched_component_start = previous_sep + 1;
        left.path = &left.path[left.normalize_front(mismatched_component_start)..];
        left.has_physical_root = false;
        right.path = &right.path[right.normalize_front(mismatched_component_start)..];
        right.has_physical_root = false;
    }

    // println!("{:?}", left.path);
    // println!("{:?}", right.path);

    // return cmp::Ordering::Less;
    Iterator::cmp(left, right)
}

fn components(path: &Path) -> Components<'_> {
    let os_str_path = path.as_os_str();
    let path_bytes = os_str_path.as_encoded_bytes();

    let mut components = Components {
        path: path_bytes,
        // Introducing Prefix causes massive performance degradation for
        // Components::next (unsure why, maybe more cache misses since this
        // struct becomes 56 bytes with Option<Prefix>?)
        // prefix: None,
        has_physical_root: has_physical_root(path_bytes),
        is_done: path_bytes.is_empty(),
    };

    components
}

#[inline]
fn eq_components(path: &Path, other: &Path) -> bool {
    path.as_os_str() == other.as_os_str() || Iterator::eq(components(path).rev(), components(other).rev())
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
    // let mut comps = Iter {
    //     inner: components(path),
    // };
    // while let Some(comp) = comps.next() {}
    let mut comps = path.components();
    while let Some(comp) = comps.next() {}
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
    // path.components() == other_path.components();
    eq_components(path, other_path);
}

fn compare_comps(path: &Path, other_path: &Path) {
    // let comp = components(path);
    // let other_comp = components(other_path);
    let comp = path.components();
    let other_comp = other_path.components();
    // println!("{:?}", comp > other_comp);
    comp > other_comp;
}

fn main() {
    let mut path = String::from("/");
    let chars = vec!["a"; 64];
    let mut str = chars.join("");
    str.push('/');

    for i in 0..1000 {
        path.push_str(&str);
    }

    // let path_b = format!("/b/{path}");
    let path_b = format!("{path}/b/");

    for i in 0..10000 {
        compare_comps(path.as_ref(), path_b.as_ref());
        // components_next_iter(path.as_ref());
    }
}
