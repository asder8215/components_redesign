#![feature(path_trailing_sep)]
#![allow(dead_code)]
#![allow(unused)]
use std::{
    cmp,
    ffi::OsStr,
    fmt,
    hash::{Hash, Hasher},
    hint::black_box,
    iter::FusedIterator,
    path::{MAIN_SEPARATOR, Path},
};

use criterion::{Criterion, criterion_group, criterion_main};

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

// fn is_sep_byte(b: u8) -> bool {
//     b == b'/'
// }

fn parse_prefix(_: &OsStr) -> Option<Prefix<'_>> {
    None
}

#[inline]
fn construct_prefix<'a>(
    _: &'a OsStr,
    _: &'a OsStr,
    _: Option<PrefixTag>,
) -> Option<Prefix<'a>> {
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

#[derive(Copy, Clone)]
pub(crate) enum PrefixTag {
    Verbatim,
    VerbatimUNC,
    VerbatimDisk,
    DeviceNS,
    UNC,
    Disk,
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
    /// The path left to parse components from
    path: &'a [u8],
    /// A tracking index to consume components from the front. If `front` starts off
    /// as non-zero on creating a `Components<'_>` iterator, a prefix is present.
    /// For UNC prefixes, this front points to the end len of the server and share
    /// portion of the prefix.
    front: usize,
    /// This is exclusively used for Windows' UNC prefixes, so that we can construct
    /// a `Prefix` through `construct_prefix()` knowing these ending indices when our
    /// first component is a Prefix. This index holds the end length of server portion
    /// of Windows' UNC prefixes.
    unc_prefix_server: usize,
    /// A tracking index to consume components from the back.`back` may not equal to
    /// `path.len()` if trailing separators are present.
    back: usize,
    /// True if path *physically* has a root separator; for most Windows
    /// prefixes, it may have a "logical" root separator for the purposes of
    /// normalization, e.g., \\server\share == \\server\share\.
    has_physical_root: bool,
    /// The first component parsed, be it a relative path (""), an absolute path ("/"),
    /// or a Prefix, which is Windows Specific
    first_comp: Option<FirstComponent>,
    /// If this is a Windows' path and we have a prefix in our path, then we will store
    /// the type of `Prefix` this path is.
    tag: Option<PrefixTag>,
}

impl<'a> Components<'a> {
    /// Is the *original* path rooted?
    fn has_root(&self) -> bool {
        if self.has_physical_root {
            return true;
        }

        // SAFETY: This u8 slice is the entire original path unmodified. The caller to
        // `Path::components` should have given us a valid `Path`.
        if HAS_PREFIXES
            && let Some(p) = parse_prefix(unsafe { OsStr::from_encoded_bytes_unchecked(self.path) })
        {
            if p.has_implicit_root() {
                return true;
            }
        }
        false
    }

    /// This returns the `Prefix` component of our `Components<'_>` iterator
    /// if it exists.
    fn get_prefix(&self) -> Option<Prefix<'a>> {
        if self.first_comp == Some(FirstComponent::Prefix) {
            // SAFETY: Our front has the length of our Prefix component encoded at the start,
            // so this slice is guaranteed to contain the Prefix components if it's
            // unconsumed.
            let unc_server_path = unsafe { OsStr::from_encoded_bytes_unchecked(&self.path[..self.unc_prefix_server]) };
            let prefix =
                unsafe { OsStr::from_encoded_bytes_unchecked(&self.path[self.unc_prefix_server..self.front]) };
            return construct_prefix(prefix, unc_server_path, self.tag);
        }
        None
    }

    /// This is a helper function for consuming the  physical first component in
    /// either `Components::next`/`Components::next_back`.
    ///
    /// There are four cases we can have here:
    /// - We have an unconsumed absolute component (`/`). We should just output `/`
    ///   in this case.
    /// - We have an unconsumed prefix component (Windows specific, e.g. `C:`).
    ///   We should just return that prefix component
    /// - We have a relative directory, we should just parse the component as
    ///   normal for the front direction only (due to 0 indexing front index)
    /// - We don't have a start component (frequent case), which means we just
    ///   return `None`.
    // #[inline]
    fn consume_first_component_front(&mut self) -> Option<Component<'a>> {
        match self.first_comp {
            Some(FirstComponent::AbsolutePath) => {
                self.first_comp = None;
                self.normalize_front();
                Some(Component::RootDir)
            }
            Some(FirstComponent::Prefix) => {
                // SAFETY: Our front has the length of our Prefix component encoded at the start,
                // so this slice is guaranteed to contain the Prefix component if it's
                // unconsumed.
                let prefix_slice =
                    unsafe { OsStr::from_encoded_bytes_unchecked(&self.path[0..self.front]) };
                // Since we know our first component is a prefix, this is safe to unwrap
                let prefix = self.get_prefix().unwrap();
                self.first_comp = None;
                self.normalize_front();
                Some(Component::Prefix(PrefixComponent { raw: prefix_slice, parsed: prefix }))
            }
            Some(FirstComponent::RelativePath) => return self.parse_next_component(),
            None => None,
        }
    }

    // #[inline]
    fn consume_first_component_back(&mut self) -> Option<Component<'a>> {
        match self.first_comp {
            Some(FirstComponent::AbsolutePath) => {
                self.first_comp = None;
                Some(Component::RootDir)
            }
            Some(FirstComponent::Prefix) => {
                // SAFETY: Our front has the length of our Prefix component encoded at the start,
                // so this slice is guaranteed to contain the Prefix component if it's
                // unconsumed.
                let prefix_slice =
                    unsafe { OsStr::from_encoded_bytes_unchecked(&self.path[0..self.front]) };
                // Since we know our first component is a prefix, this is safe to unwrap
                let prefix = self.get_prefix().unwrap();
                self.first_comp = None;
                Some(Component::Prefix(PrefixComponent { raw: prefix_slice, parsed: prefix }))
            }
            _ => None,
        }
    }

    /// Normalizes away trailing separators and current directory ('.') components
    /// in the forward direction.
    // #[inline]
    fn normalize_front(&mut self) {
        let path = &self.path[self.front..self.back];
        // ".a", ".." needs to rebound back to index
        // before the "." character
        let mut cur_dir_present = false;
        match path.iter().position(|b| {
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
            None => self.front = self.back,
            Some(i) => {
                if cur_dir_present {
                    self.front += i - 1;
                } else {
                    self.front += i;
                }
            }
        }
    }

    /// Normalizes away trailing separators and current directory ('.') components
    /// in the backward direction.
    // #[inline]
    fn normalize_back(&mut self) {
        let path = &self.path[self.front..self.back];
        // "a.", ".." needs to rebound back to index
        // before the "." character
        let mut cur_dir_present = false;
        match path.iter().rposition(|b| {
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
                    self.back = self.front + 1;
                } else {
                    self.back = self.front;
                }
            }
            Some(i) => {
                if cur_dir_present {
                    self.back -= path.len() - i - 2;
                } else {
                    self.back -= path.len() - i - 1;
                }
            }
        }
    }

    /// Increments our front pointer until we find the
    /// next separator byte or have reached the component
    /// that back index is pointing at.
    // #[inline]
    fn find_next_separator_front(&mut self) {
        let path = &self.path[self.front..self.back];
        match path.iter().position(|b| is_sep_byte(*b)) {
            None => self.front = self.back,
            Some(i) => self.front += i + 1,
        }
    }

    /// Decrements our back pointer until we find the
    /// next separator byte or have reached the component
    /// that front index is pointing to.
    // #[inline]
    fn find_next_separator_back(&mut self) {
        let path = &self.path[self.front..self.back];
        match path.iter().rposition(|b| is_sep_byte(*b)) {
            None => self.back = self.front,
            Some(i) => self.back -= path.len() - i,
        }
    }

    /// Parse a u8 slice into an OsStr, which is encoded into a `Component`
    // #[inline]
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
        match self.first_comp {
            Some(FirstComponent::AbsolutePath) => {
                // If back index is at 0 (e.g parsing backward
                // through "/foo") and we have an unconsumed
                // Root component, Components::as_path needs to
                // return "/" path
                if self.back == 0 {
                    return Path::new("/");
                }
            }
            Some(FirstComponent::Prefix) => {
                // We don't want to trim away separators from a Prefix
                // component
                if self.front == self.back {
                    // SAFETY: If the first component is not consumed, then
                    // front index encodes the whole length of the Prefix
                    // component
                    return unsafe { from_u8_slice(&self.path[..self.front]) };
                }
                // SAFETY: Our back index is guaranteed to delimit at an ascii
                // separator byte, so this should present a valid path
                return unsafe { from_u8_slice(&self.path[..self.back]).trim_trailing_sep() };
            }
            _ => {}
        }
        // SAFETY: front and back index are delimited by ascii separator bytes,
        // where front is a byte after an ascii separator and back is at an ascii
        // separator, so this will always produce a valid path.
        unsafe { from_u8_slice(&self.path[self.front..self.back]).trim_trailing_sep() }
    }

    /// Parses the next component in `Components<'_>` from the left
    // #[inline]
    fn parse_next_component(&mut self) -> Option<Component<'a>> {
        // Our current `self.front` index at this point is the start
        // of the component name
        let before_front = self.front;
        // We trace our `self.front` idx down the path until
        // we hit a separator.
        self.find_next_separator_front();
        let curr_front = self.front;
        // Normalizes trailing seps and curr dirs in preparation for
        // next front component
        self.normalize_front();

        // SAFETY: Our curr_front index always stops a byte after the ascii
        // separator byte or at self.back (should there be no ascii separator
        // in traversal), so we can always construct a valid u8 path slice
        let sliced_path = if curr_front > 0 && is_sep_byte(self.path[curr_front - 1]) {
            &self.path[before_front..curr_front - 1]
        } else {
            &self.path[before_front..curr_front]
        };
        self.parse_single_component(sliced_path)
    }

    /// Parses the next back component in `Components<'_>` from the
    /// right
    // #[inline]
    fn parse_next_back_component(&mut self) -> Option<Component<'a>> {
        // Our current `self.back` index at this point encompasses
        // the parent path
        let before_back = self.back;
        // We trace our `self.back` idx up the path until we reach a
        // separator byte. This prepares the path we return on the next
        // call to this function.
        self.find_next_separator_back();
        let curr_back = self.back;
        // Normalizes trailing seps and curr dirs in preparation for
        // next back component
        self.normalize_back();

        // Our curr_back is at the byte before an ascii separator byte or self.front,
        // (should there be no ascii separator in traversal), so we can always
        // construct a valid u8 path slice
        let sliced_path = if is_sep_byte(self.path[curr_back]) {
            &self.path[curr_back + 1..before_back]
        } else {
            &self.path[curr_back..before_back]
        };
        self.parse_single_component(sliced_path)
    }
}

impl<'a> Iterator for Components<'a> {
    type Item = Component<'a>;

    // #[inline]
    fn next(&mut self) -> Option<Component<'a>> {
        // We reach this case when we no longer have anymore paths
        // to consume (return `None`), or if our front idx was initially
        // equal to back idx (e.g. if we had `C:`, `.`, `/`)
        if self.front >= self.back || self.first_comp.is_some() {
            return self.consume_first_component_front();
        }

        self.parse_next_component()
    }
}

impl<'a> DoubleEndedIterator for Components<'a> {
    // #[inline]
    fn next_back(&mut self) -> Option<Component<'a>> {
        // We reach here when we no longer have anymore paths
        // to consume, we're dealing with relative paths and
        // need to output "", or we need to output Prefix component
        if self.back <= self.front {
            return self.consume_first_component_back();
        }

        self.parse_next_back_component()
    }
}

impl FusedIterator for Components<'_> {}

impl<'a> PartialEq for Components<'a> {
    // #[inline]
    fn eq(&self, other: &Components<'a>) -> bool {
        // Fast path for exact matches, e.g. for hashmap lookups.
        // Don't explicitly compare the prefix or has_physical_root fields since they'll
        // either be covered by the `path` buffer or are only relevant for `prefix_verbatim()`.
        if self.path.len() == other.path.len()
            && self.front == other.front
            && self.back == other.back
        {
            // possible future improvement: this could bail out earlier if there were a
            // reverse memcmp/bcmp comparing back to front

            // If either `self` or `other` have a prefix (indicated by `first_comp`)
            // we need to start at index 0 (because prefix length is encoded in
            // `front`)
            let path = if matches!(self.first_comp, Some(FirstComponent::Prefix)) {
                &self.path[..self.back]
            } else {
                &self.path[self.front..self.back]
            };

            let other_path = if matches!(other.first_comp, Some(FirstComponent::Prefix)) {
                &other.path[..other.back]
            } else {
                &other.path[other.front..other.back]
            };
            if path == other_path {
                return true;
            }
        }

        // // compare back to front since absolute paths often share long prefixes
        Iterator::eq(self.clone().rev(), other.clone().rev())
        // compare_components(self.clone(), other.clone()) == cmp::Ordering::Equal
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
    let path = if let Some(p) = prefix {
        &s[p.len()..]
    } else {
        s
    };
    !path.is_empty() && is_sep_byte(path[0])
}

fn compare_components(mut left: Components<'_>, mut right: Components<'_>) -> cmp::Ordering {
    // Fast path for long shared prefixes
    //
    // - compare raw bytes to find first mismatch
    // - backtrack to find separator before mismatch to avoid ambiguous parsings of '.' or '..' characters
    // - if found update state to only do a component-wise comparison on the remainder,
    //   otherwise do it on the full path
    //
    // The fast path isn't taken for paths with a PrefixComponent to avoid backtracking into
    // the middle of one. If both left and right are at 0, that means no prefix was encoded
    // into this
    // possible future improvement: a [u8]::first_mismatch simd implementation
    // Optimization: can check if the differing character is not a '/' or '.'
    // and then return either `Ordering::Greater` or `Ordering::Less`

    // let mut left_path = if matches!(left.first_comp, Some(FirstComponent::Prefix)) {
    //     &left.path[..left.back]
    // } else {
    //     &left.path[left.front..left.back]
    // };

    // let mut right_path = if matches!(right.first_comp, Some(FirstComponent::Prefix)) {
    //     &right.path[..right.back]
    // } else {
    //     &right.path[right.front..right.back]
    // };

    // loop {
    //     match left_path.iter().zip(right_path).position(
    //         |(&a, &b)| {
    //             a != b || (a == MAIN_SEPARATOR as u8 && b == MAIN_SEPARATOR as u8)
    //         }
    //     ) {
    //         None if left_path.len() == right_path.len() => return cmp::Ordering::Equal,
    //         None => return left_path.len().cmp(&right_path.len()),
    //         Some(pos) => {
    //             let left_byte = left_path[pos];
    //             let right_byte = right_path[pos];
    //             if left_byte == MAIN_SEPARATOR as u8 && right_byte == MAIN_SEPARATOR as u8 {
    //                 let normalize_left_path = &left_path[pos..];
    //                 let normalize_right_path = &right_path[pos..];
    //                 // ".a", ".." needs to rebound back to index
    //                 // before the "." character
    //                 let mut cur_dir_present = false;
    //                 match normalize_left_path.iter().position(|b| {
    //                     if !is_sep_byte(*b) {
    //                         if *b == b'.' && !cur_dir_present {
    //                             cur_dir_present = true;
    //                             false
    //                         } else {
    //                             true
    //                         }
    //                     } else {
    //                         cur_dir_present = false;
    //                         false
    //                     }
    //                 }) {
    //                     None => left_path = &[],
    //                     Some(i) => {
    //                         if cur_dir_present {
    //                             left_path =  &normalize_left_path[i - 1..];
    //                         } else {
    //                             left_path = &normalize_left_path[i..];
    //                         }
    //                     }
    //                 }
    //                 cur_dir_present = false;
    //                 match normalize_right_path.iter().position(|b| {
    //                     if !is_sep_byte(*b) {
    //                         if *b == b'.' && !cur_dir_present {
    //                             cur_dir_present = true;
    //                             false
    //                         } else {
    //                             true
    //                         }
    //                     } else {
    //                         cur_dir_present = false;
    //                         false
    //                     }
    //                 }) {
    //                     None => left_path = &[],
    //                     Some(i) => {
    //                         if cur_dir_present {
    //                             right_path =  &normalize_right_path[i - 1..];
    //                         } else {
    //                             right_path = &normalize_right_path[i..];
    //                         }
    //                     }
    //                 }
    //             } else if left_byte == MAIN_SEPARATOR as u8 {
    //                 return cmp::Ordering::Less;
    //             } else if right_byte == MAIN_SEPARATOR as u8 {
    //                 return cmp::Ordering::Greater;
    //             } else {
    //                 return left_byte.cmp(&right_byte);
    //             }
    //         }
    //     }
    // }

    // if left.front == 0 && right.front == 0 {
    //     // Note: This is one of the strangest things I've noticed
    //     // through benchmarking the `None` matches, in the default
    //     // `None` case, if I compare `left.back.min(right.back)`
    //     // it actually makes benchmarking slower than using
    //     // these two variables left_back.min(right_back)
    //     // I need someone to explain why this occurs
    //     let left_back = left.back;
    //     let right_back = right.back;
    //     let first_difference = match left.path[..left.back]
    //         .iter()
    //         .zip(&right.path[..right.back])
    //         .position(|(&a, &b)| a != b)
    //     {
    //         None if left.back == right.back => return cmp::Ordering::Equal,
    //         None => left_back.min(right_back),
    //         Some(diff) => diff,
    //     };
    //     if let Some(previous_sep) = left.path[..first_difference]
    //         .iter()
    //         .rposition(|&b| is_sep_byte(b))
    //     {
    //         // We should always set first_comp to `None` since we got past
    //         // the first character (could be root dir or a part of a relative path)
    //         // we normalize both `Components<'_>` because we want both to start
    //         // at a non-separator character and start comparing from there
    //         // (e.g. comparing "/a" with "///a")
    //         left.first_comp = None;
    //         left.front = previous_sep;
    //         left.normalize_front();
    //         right.first_comp = None;
    //         right.front = previous_sep;
    //         right.normalize_front();
    //     }
    // }

    // Iterator::cmp(left, right)

    if let Some(left_first_comp) = left.first_comp
        && let Some(right_first_comp) = right.first_comp
    {
        match (left_first_comp, right_first_comp) {
            (FirstComponent::AbsolutePath, FirstComponent::RelativePath) => {
                if right.back > 0 {
                    return left.path[0].cmp(&right.path[0]);
                }
                return cmp::Ordering::Greater;
            }
            (FirstComponent::RelativePath, FirstComponent::AbsolutePath) => {
                if left.back > 0 {
                    return left.path[0].cmp(&right.path[0]);
                }
                return cmp::Ordering::Less;
            }
            (FirstComponent::AbsolutePath, FirstComponent::AbsolutePath)
            | (FirstComponent::RelativePath, FirstComponent::RelativePath) => {}
            _ => return Iterator::cmp(left, right),
        }
    }

    let mut left_front = left.front;
    let mut right_front = right.front;
    let left_back = left.back;
    let right_back = right.back;

    loop {
        match left.path[left_front..left_back]
            .iter()
            .zip(right.path[right_front..right_back].iter())
            .position(|(&a, &b)| a != b)
        // match std::iter::zip(left.path[left_front..left_back].iter(),right.path[right_front..right_back].iter()).position(|(&a, &b)| a != b)
        {
            None if left_back - left_front == right_back - right_front => {
                // println!("hi");
                return cmp::Ordering::Equal;
            }
            None => {
                let mut cur_dir_present = false;
                if left_back - left_front > right_back - right_front {
                    if right_back > 0
                        && left.path[right_back] == b'.'
                        && left.path[right_back - 1] != MAIN_SEPARATOR as u8
                    {
                        return cmp::Ordering::Greater;
                    }
                    match left.path[right_back..left_back].iter().position(|b| {
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
                        None => return cmp::Ordering::Equal,
                        Some(i) => return cmp::Ordering::Greater,
                    }
                } else {
                    if left_back > 0
                        && right.path[left_back] == b'.'
                        && right.path[left_back - 1] != MAIN_SEPARATOR as u8
                    {
                        return cmp::Ordering::Less;
                    }
                    match right.path[left_back..right_back].iter().position(|b| {
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
                        None => return cmp::Ordering::Equal,
                        Some(i) => return cmp::Ordering::Less,
                    }
                }
            }
            Some(ind) => {
                left_front += ind;
                right_front += ind;
                let left_byte = left.path[left_front];
                let right_byte = right.path[right_front];
                // a/b/c/./././d
                // a/b/c/d
                if left_byte == MAIN_SEPARATOR as u8 && right_byte != MAIN_SEPARATOR as u8 {
                    let mut cur_dir_present = false;
                    match left.path[left_front..left_back].iter().position(|b| {
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
                        None => return cmp::Ordering::Less,
                        Some(i) => {
                            if cur_dir_present {
                                left_front += i - 1;
                            } else {
                                left_front += i;
                            }
                        }
                    }
                } else if left_byte != MAIN_SEPARATOR as u8 && right_byte == MAIN_SEPARATOR as u8 {
                    let mut cur_dir_present = false;
                    match right.path[right_front..right_back].iter().position(|b| {
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
                        None => return cmp::Ordering::Greater,
                        Some(i) => {
                            if cur_dir_present {
                                right_front += i - 1;
                            } else {
                                right_front += i;
                            }
                        }
                    }
                } else {
                    if left_byte == b'.' || right_byte == b'.' {
                        break;
                    }
                    return left_byte.cmp(&right_byte);
                }
            }
        }
    }

    // loop {
    //     let left_byte = left_path.next();
    //     let right_byte = right_path.next();
    //     left_front += 1;
    //     right_front += 1;

    //     match (left_byte, right_byte) {
    //         (None, None) => return cmp::Ordering::Equal,
    //         (None, Some(right_byte)) => {
    //             let right_byte = *right_byte;
    //             if right_byte == MAIN_SEPARATOR as u8 || right_byte == b'.' {
    //                 let mut cur_dir_present = false;
    //                 match right.path[left_back..right_back].iter().position(|b| {
    //                     if !is_sep_byte(*b) {
    //                         if *b == b'.' && !cur_dir_present {
    //                             cur_dir_present = true;
    //                             false
    //                         } else {
    //                             true
    //                         }
    //                     } else {
    //                         cur_dir_present = false;
    //                         false
    //                     }
    //                 }) {
    //                     None => return cmp::Ordering::Equal,
    //                     Some(i) => {},
    //                 }
    //             }
    //             return cmp::Ordering::Less;
    //         },
    //         (Some(left_byte), None) => {
    //             let left_byte = *left_byte;
    //             if left_byte == MAIN_SEPARATOR as u8 || left_byte == b'.' {
    //                 let mut cur_dir_present = false;
    //                 match left.path[right_back..left_back].iter().position(|b| {
    //                     if !is_sep_byte(*b) {
    //                         if *b == b'.' && !cur_dir_present {
    //                             cur_dir_present = true;
    //                             false
    //                         } else {
    //                             true
    //                         }
    //                     } else {
    //                         cur_dir_present = false;
    //                         false
    //                     }
    //                 }) {
    //                     None => return cmp::Ordering::Equal,
    //                     Some(i) => {}
    //                 }
    //             }
    //             return cmp::Ordering::Greater;
    //         },
    //         (Some(left_byte), Some(right_byte)) => {
    //             let left_byte = *left_byte;
    //             let right_byte = *right_byte;
    //             if left_byte == MAIN_SEPARATOR as u8 && right_byte != MAIN_SEPARATOR as u8 {
    //                 let mut cur_dir_present = false;
    //                 match left.path[left_front..left_back].iter().position(|b| {
    //                     if !is_sep_byte(*b) {
    //                         if *b == b'.' && !cur_dir_present {
    //                             cur_dir_present = true;
    //                             false
    //                         } else {
    //                             true
    //                         }
    //                     } else {
    //                         cur_dir_present = false;
    //                         false
    //                     }
    //                 }) {
    //                     None => return cmp::Ordering::Less,
    //                     Some(i) => {
    //                         if cur_dir_present {
    //                             left_front += i - 1;
    //                         } else {
    //                             left_front += i;
    //                         }
    //                         left_path = left.path[left_front..left_back].iter();
    //                     },
    //                 }
    //             } else if left_byte != MAIN_SEPARATOR as u8 && right_byte == MAIN_SEPARATOR as u8 {
    //                 let mut cur_dir_present = false;
    //                 match right.path[right_front..right_back].iter().position(|b| {
    //                     if !is_sep_byte(*b) {
    //                         if *b == b'.' && !cur_dir_present {
    //                             cur_dir_present = true;
    //                             false
    //                         } else {
    //                             true
    //                         }
    //                     } else {
    //                         cur_dir_present = false;
    //                         false
    //                     }
    //                 }) {
    //                     None => return cmp::Ordering::Greater,
    //                     Some(i) => {
    //                         if cur_dir_present {
    //                             right_front += i - 1;
    //                         } else {
    //                             right_front += i;
    //                         }
    //                         right_path = right.path[right_front..right_back].iter();
    //                     },
    //                 }
    //             } else if left_byte == b'.' || right_byte == b'.' {
    //                     break;
    //             } else if left_byte > right_byte {
    //                 return cmp::Ordering::Greater;
    //             } else if left_byte < right_byte {
    //                 return cmp::Ordering::Less;
    //             }
    //         },
    //     }
    // }

    if let Some(left_previous_sep) = left.path[..left_front]
        .iter()
        .rposition(|&b| is_sep_byte(b))
        && let Some(right_prev_sep) = right.path[..right_front]
            .iter()
            .rposition(|&b| is_sep_byte(b))
    {
        left.first_comp = None;
        left.front = left_previous_sep;
        left.normalize_front();
        right.first_comp = None;
        right.front = right_prev_sep;
        right.normalize_front();
    }

    Iterator::cmp(left, right)
}

fn components(path: &Path) -> Components<'_> {
    let os_str_path = path.as_os_str();
    let path_bytes = os_str_path.as_encoded_bytes();

    // Windows specific component
    let prefix = parse_prefix(os_str_path);
    let prefix_exist = prefix.map(|_| true).unwrap_or(false);

    let has_physical_root = has_physical_root(path_bytes, prefix);
    let first_comp = if prefix_exist {
        Some(FirstComponent::Prefix)
    } else if has_physical_root {
        Some(FirstComponent::AbsolutePath)
    } else {
        Some(FirstComponent::RelativePath)
    };

    // If we have a prefix, we encode that index into front as well as the tag.
    let (tag, unc_prefix_server, front) = prefix
        .map(|prefix| match prefix {
            Prefix::DeviceNS(_) => (Some(PrefixTag::DeviceNS), 0, prefix.len()),
            Prefix::Disk(_) => (Some(PrefixTag::Disk), 0, prefix.len()),
            Prefix::UNC(server, _) => (Some(PrefixTag::UNC), server.len(), prefix.len()),
            Prefix::Verbatim(_) => (Some(PrefixTag::Verbatim), 0, prefix.len()),
            Prefix::VerbatimDisk(_) => (Some(PrefixTag::VerbatimDisk), 0, prefix.len()),
            Prefix::VerbatimUNC(server, _) => {
                (Some(PrefixTag::VerbatimUNC), server.len(), prefix.len())
            }
        })
        .unwrap_or((None, 0, 0));
    let back = path_bytes.len();

    let mut components = Components {
        path: path_bytes,
        has_physical_root,
        front,
        unc_prefix_server,
        back,
        first_comp,
        tag,
    };

    // Normalize any trailing separators or cur dir (".") components away
    components.normalize_back();

    components
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

fn create_components(path: &Path) {
    let comp = path.components();
}

fn components_iter(path: &Path) {
    let comps = components(path);
    for comp in comps {}
}

fn components_next_iter(path: &Path) {
    let mut comps = Iter {
        inner: components(path),
    };
    while let Some(comp) = comps.next() {}
}

fn components_next_back_iter(path: &Path) {
    let mut comps = Iter {
        inner: components(path),
    };
    while let Some(comp) = comps.next_back() {}
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
    let comp = components(path);
    let other_comp = components(other_path);
    comp == other_comp;
}

fn compare_comps(path: &Path, other_path: &Path) {
    let comp = components(path);
    let other_comp = components(other_path);
    comp > other_comp;
}

fn bench_components_fast(c: &mut Criterion) {
    let mut path = String::from("/");
    let chars = vec!["a"; 64];
    let mut str = chars.join("");
    str.push('/');

    for i in 0..1000 {
        path.push_str(&str);
    }

    // "/a0..a64/a0..a64/a0..a64/.../b/"
    let path_b = format!("{path}/b/");

    // "/b/a0..a64/a0..a64/.../a0..a64/"
    let path_c = format!("/b/{path}");

    let path_d = format!("{path}/b/{path}"); 

    let path_e = format!("{path}{path}");

    c.bench_function("Create Components Rewrite", |b| {
        b.iter(|| black_box(create_components(black_box(path.as_ref()))))
    });

    c.bench_function("Components Rewrite", |b| {
        b.iter(|| black_box(components_iter(black_box(path.as_ref()))))
    });

    c.bench_function("Components Next Rewrite", |b| {
        b.iter(|| black_box(components_next_iter(black_box(path.as_ref()))))
    });

    c.bench_function("Components Next Back Rewrite", |b| {
        b.iter(|| black_box(components_next_back_iter(black_box(path.as_ref()))))
    });

    c.bench_function("Path Iter Rewrite", |b| {
        b.iter(|| black_box(path_iter(black_box(path.as_ref()))))
    });

    c.bench_function("As Path Iter Rewrite", |b| {
        b.iter(|| black_box(as_path_iter(black_box(path.as_ref()))))
    });

    c.bench_function("Eq Comps Rewrite", |b| {
        b.iter(|| black_box(eq_comps(black_box(path.as_ref()), black_box(path.as_ref()))))
    });

    c.bench_function("Uneq Comps Rewrite", |b| {
        b.iter(|| {
            black_box(eq_comps(
                black_box(path.as_ref()),
                black_box(path_b.as_ref()),
            ))
        })
    });

    c.bench_function("Uneq 2 Comps Rewrite", |b| {
        b.iter(|| {
            black_box(eq_comps(
                black_box(path.as_ref()),
                black_box(path_c.as_ref()),
            ))
        })
    });

    c.bench_function("Compare Comps Rewrite", |b| {
        b.iter(|| {
            black_box(compare_comps(
                black_box(path.as_ref()),
                black_box(path.as_ref()),
            ))
        })
    });

    c.bench_function("Compare Uneq Comps Rewrite", |b| {
        b.iter(|| {
            black_box(compare_comps(
                black_box(path.as_ref()),
                black_box(path_b.as_ref()),
            ))
        })
    });

    c.bench_function("Compare Uneq 2 Comps Rewrite", |b| {
        b.iter(|| {
            black_box(compare_comps(
                black_box(path.as_ref()),
                black_box(path_c.as_ref()),
            ))
        })
    });

    c.bench_function("Compare Uneq 3 Comps Rewrite", |b| {
        b.iter(|| {
            black_box(compare_comps(
                black_box(path_d.as_ref()),
                black_box(path_e.as_ref()),
            ))
        })
    });

    // ----------- WITHOUT BLACK BOX ---------------------

    // c.bench_function("Components Rewrite (No BB)", |b| {
    //     b.iter(|| {
    //         components_iter(path.as_ref())
    //     })
    // });

    // c.bench_function("Components Next Rewrite (No BB)", |b| {
    //     b.iter(|| {
    //         components_next_iter(path.as_ref())
    //     })
    // });

    // c.bench_function("Components Next Back Rewrite (No BB)", |b| {
    //     b.iter(|| {
    //         components_next_back_iter(path.as_ref())
    //     })
    // });

    // c.bench_function("Path Iter Rewrite (No BB)", |b| {
    //     b.iter(|| {
    //         path_iter(path.as_ref())
    //     })
    // });

    // c.bench_function("As Path Iter Rewrite (No BB)", |b| {
    //     b.iter(|| {
    //         as_path_iter(path.as_ref())
    //     })
    // });

    // c.bench_function("Eq Comps Rewrite (No BB)", |b| {
    //     b.iter(|| {
    //         eq_comps(path.as_ref(), path.as_ref())
    //     })
    // });

    // c.bench_function("Uneq Comps Rewrite (No BB)", |b| {
    //     b.iter(|| {
    //         eq_comps(path.as_ref(), path_b.as_ref())
    //     })
    // });

    // c.bench_function("Uneq Comps 2 Rewrite (No BB)", |b| {
    //     b.iter(|| {
    //         eq_comps(path.as_ref(), path_c.as_ref())
    //     })
    // });

    // c.bench_function("Uneq Comps 3 Rewrite (No BB)", |b| {
    //     b.iter(|| {
    //         eq_comps(path_d.as_ref(), path_e.as_ref())
    //     })
    // });

    // c.bench_function("Compare Comps Rewrite (No BB)", |b| {
    //     b.iter(|| compare_comps(path.as_ref(), path.as_ref()))
    // });

    // c.bench_function("Compare Uneq Comps Rewrite (No BB)", |b| {
    //     b.iter(|| compare_comps(path.as_ref(), path_b.as_ref()))
    // });

    // c.bench_function("Compare Uneq Comps 2 Rewrite (No BB)", |b| {
    //     b.iter(|| compare_comps(path.as_ref(), path_c.as_ref()))
    // });
}

criterion_group!(benches, bench_components_fast);
criterion_main!(benches);
