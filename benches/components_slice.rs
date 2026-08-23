#![allow(dead_code)]
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

#[allow(unused)]
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
    #[allow(unused)]
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

    #[allow(unused)]
    #[inline]
    fn is_drive(&self) -> bool {
        matches!(*self, Prefix::Disk(_))
    }

    #[allow(unused)]
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

/// Component parsing works by a double-ended state machine; the cursors at the
/// front and back of the path each keep track of what parts of the path have
/// been consumed so far.
///
/// Going front to back, a path is made up of a prefix, a starting
/// directory component, and a body (of normal components)
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
enum State {
    Absolute = 1, // A root component (i.e. '/')
    Relative = 2, // A relative component ('foo')
    Done = 3,     // Iterator is fully consumed
}

#[derive(Clone)]
pub struct Components<'a> {
    // The path left to parse components from
    path: &'a [u8],
    // The current state of the iterator
    state: State,
}

impl<'a> Components<'a> {
    /// Checks if all bytes of our path have been consumed
    #[inline]
    fn is_done(&self) -> bool {
        self.state == State::Done
    }

    /// Is the *original* path rooted?
    #[inline]
    fn has_root(&self) -> bool {
        self.state == State::Absolute
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
                self.state = State::Done;
                return self.path.len();
            }
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
                    1
                } else {
                    self.state = State::Done;
                    0
                }
            }
            Some(i) => {
                if cur_dir_present {
                    i + 2
                } else {
                    i + 1
                }
            }
        }
    }

    /// Parse a u8 slice into an OsStr, which is encoded into a `Component`
    #[inline]
    fn parse_single_component(&self, slice: &'a [u8]) -> Option<Component<'a>> {
        match slice {
            [] => None,
            [b'.'] => Some(Component::CurDir),
            [b'.', b'.'] => Some(Component::ParentDir),
            _ => {
                let root_slice = MAIN_SEPARATOR_STR.as_bytes();
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
        // Normalize bytes from the right
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
        // SAFETY: Back should be at a separator byte (or index 0 if
        // no separator byte exist), which slicing path_bytes at that index
        // should give us a valid slice
        unsafe { from_u8_slice(&self.path[..back]) }
    }

    /// Parses the next component in `Components<'_>` from the left
    #[inline]
    fn parse_next_component(&mut self) -> (usize, Option<Component<'a>>) {
        let (front_ind, comp) = match self.path.iter().position(|b| is_sep_byte(*b)) {
            None => (self.path.len(), self.path),
            Some(i) => (i + 1, &self.path[..i]),
        };

        (front_ind, self.parse_single_component(comp))
    }

    /// Parses the next back component in `Components<'_>` from the
    /// right
    #[inline]
    fn parse_next_back_component(&mut self, back: usize) -> (usize, Option<Component<'a>>) {
        let (back_ind, comp) = match self.path[..back].iter().rposition(|b| is_sep_byte(*b)) {
            None => {
                self.state = State::Done;
                (0, &self.path[..back])
            }
            Some(i) => (i, &self.path[i + 1..back]),
        };

        (back_ind, self.parse_single_component(comp))
    }
}

impl<'a> Iterator for Components<'a> {
    type Item = Component<'a>;

    #[inline]
    fn next(&mut self) -> Option<Component<'a>> {
        // Changing this to a pure match case body with State::Absolute,
        // State::Relative, State::Done causes performance degradation
        // with `Components` ordering. Unsure why, but writing the code like
        // this maintains performance on par with the prefixed version.
        if !self.is_done() {
            match self.state {
                State::Absolute => {
                    let end_ind = self.normalize_front(0);
                    self.path = if self.is_done() {
                        &[]
                    } else {
                        self.state = State::Relative;
                        &self.path[end_ind..]
                    };

                    if !self.is_done() {
                        self.state = State::Relative;
                    }

                    return Some(Component::RootDir);
                }
                _ => {
                    let (front_ind, comp) = self.parse_next_component();
                    let normalized_front_ind = self.normalize_front(front_ind);
                    self.path = if self.is_done() {
                        &[]
                    } else {
                        &self.path[normalized_front_ind..]
                    };
                    return comp;
                }
            }
        }
        None
    }
}

impl<'a> DoubleEndedIterator for Components<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Component<'a>> {
        if !self.is_done() {
            let back = self.normalize_back();
            if self.is_done() {
                self.path = &[];
                return Some(Component::RootDir);
            } else {
                let (back_ind, comp) = self.parse_next_back_component(back);
                self.path = &self.path[..back_ind];
                return comp;
            }
        }

        None
    }
}

impl FusedIterator for Components<'_> {}

impl<'a> PartialEq for Components<'a> {
    #[inline]
    fn eq(&self, other: &Components<'a>) -> bool {
        // Fast path for exact matches, e.g. for hashmap lookups.
        if self.path == other.path {
            return true;
        }

        eq_components(self.clone(), other.clone())
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

#[inline]
fn eq_components(mut left: Components<'_>, mut right: Components<'_>) -> bool {
    // One of them is an empty path
    if left.is_done() != left.is_done() {
        return false;
    }

    // Both are empty paths
    if left.is_done() && right.is_done() {
        return true;
    }

    let (left_diff, right_diff) = match left
        .path
        .iter()
        .rev()
        .zip(right.path.iter().rev())
        .enumerate()
        .find(|(_, (a, b))| a != b)
    {
        None => {
            let bytes_consumed = left.path.len().min(right.path.len());
            (
                left.path.len() - bytes_consumed,
                right.path.len() - bytes_consumed,
            )
        }
        Some((index, (a, b))) => {
            let a = *a;
            let b = *b;
            if a != b {
                if a != MAIN_SEPARATOR as u8 && a != b'.' && b != MAIN_SEPARATOR as u8 && b != b'.'
                {
                    return false;
                }
            }

            (left.path.len() - index - 1, right.path.len() - index - 1)
        }
    };

    // Cases like "foo/./bar" == "foo/bar", "foobar/bar" == "foobar", needs to consider
    // whether left_diff/right_diff is at a separator byte or not.
    if left.path[left_diff] != MAIN_SEPARATOR as u8 {
        if let Some(next_sep) = left.path[left_diff..].iter().position(|&b| is_sep_byte(b)) {
            left.path = &left.path[..left_diff + next_sep];
            right.path = &right.path[..right_diff + next_sep];
        }
    } else if right.path[right_diff] != MAIN_SEPARATOR as u8 {
        if let Some(next_sep) = right.path[right_diff..]
            .iter()
            .position(|&b| is_sep_byte(b))
        {
            left.path = &left.path[..left_diff + next_sep];
            right.path = &right.path[..right_diff + next_sep];
        }
    } else {
        left.path = &left.path[..left_diff];
        right.path = &right.path[..right_diff];
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

    if left.path == right.path {
        return cmp::Ordering::Equal;
    }

    let first_difference = match left
        .path
        .iter()
        .zip(right.path)
        .enumerate()
        .find(|(_, (a, b))| a != b)
    {
        None => left.path.len().min(right.path.len()),
        Some((index, (a, b))) => {
            let a = *a;
            let b = *b;

            if a != MAIN_SEPARATOR as u8 && a != b'.' && b != MAIN_SEPARATOR as u8 && b != b'.' {
                return a.cmp(&b);
            }

            index
        }
    };

    if let Some(previous_sep) = left.path[..first_difference]
        .iter()
        .rposition(|&b| is_sep_byte(b))
    {
        // If our state initially started as an absolute path, the root component
        // is guaranteed to be sliced away, so treat the state as if it were a
        // relative component
        let mismatched_component_start = previous_sep + 1;
        left.state = State::Relative;
        right.state = State::Relative;
        left.path = &left.path[left.normalize_front(mismatched_component_start)..];
        right.path = &right.path[right.normalize_front(mismatched_component_start)..];
    }

    Iterator::cmp(left, right)
}

#[inline]
fn components(path: &Path) -> Components<'_> {
    let os_str_path = path.as_os_str();
    let path = os_str_path.as_encoded_bytes();

    let state = if path.is_empty() {
        State::Done
    } else if is_sep_byte(path[0]) {
        State::Absolute
    } else {
        State::Relative
    };

    Components { path, state }
}

// This is unused
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

#[allow(dead_code)]
fn components_iter(path: &Path) {
    let comps = components(path);
    for _ in comps {}
}

fn components_next_iter(path: &Path) {
    for _ in 0..100 {
        let mut comps = components(path);
        while let Some(_) = comps.next() {}
    }
}

fn components_next_back_iter(path: &Path) {
    for _ in 0..100 {
        let mut comps = components(path);
        while let Some(_) = comps.next_back() {}
    }
}

#[allow(dead_code)]
fn path_iter(path: &Path) {
    let comps = Iter {
        inner: components(path),
    };
    for _ in comps {}
}

fn as_path_iter(path: &Path) {
    for _ in 0..100 {
        let mut comps = components(path);
        while let Some(_) = comps.next() {
            let _ = comps.as_path();
        }
    }
}

#[allow(unused_must_use)]
fn eq_comps(path: &Path, other_path: &Path) {
    for _ in 0..100 {
        // components(path) == components(other_path);
        _eq_comps(path, other_path);
    }
}

#[allow(unused_must_use)]
fn compare_comps(path: &Path, other_path: &Path) {
    for _ in 0..100 {
        let comp = components(path);
        let other_comp = components(other_path);
        comp > other_comp;
    }
}

#[allow(dead_code)]
fn std_components_iter(path: &Path) {
    for _ in 0..100 {
        let comps = path.components();
        for _ in comps {}
    }
}

fn std_components_next_iter(path: &Path) {
    for _ in 0..100 {
        let mut comps = path.components();
        while let Some(_) = comps.next() {}
    }
}

fn std_components_next_back_iter(path: &Path) {
    for _ in 0..100 {
        let mut comps = path.components();
        while let Some(_) = comps.next_back() {}
    }
}

#[allow(dead_code)]
fn std_path_iter(path: &Path) {
    for _ in 0..100 {
        let comps = path.iter();
        for _ in comps {}
    }
}

fn std_as_path_iter(path: &Path) {
    for _ in 0..100 {
        let mut comps = path.iter();
        while let Some(_) = comps.next() {
            let _ = comps.as_path();
        }
    }
}

#[allow(unused_must_use)]
fn std_eq_comps(path: &Path, other_path: &Path) {
    for _ in 0..100 {
        path.components() == other_path.components();
    }
}

#[allow(unused_must_use)]
fn std_compare_comps(path: &Path, other_path: &Path) {
    for _ in 0..100 {
        let comp = path.components();
        let other_comp = other_path.components();
        comp > other_comp;
    }
}

fn bench_components_fast(c: &mut Criterion) {
    // maximum bytes for a file name on Linux,
    // we'll use this as an ideal limit on what a long
    // path component looks like
    const NAME_MAX: usize = 255;
    // path max on Linux, we'll use this as an ideal
    // limit on what a long path should be
    const PATH_MAX: usize = 4096;

    let mut path_strings = vec![];
    // let short_comp = vec!["a/"].join("");
    let mut long_comp = vec!["a"; NAME_MAX].join("");
    long_comp.push('/');

    let mut comp_len = 2;

    while comp_len <= NAME_MAX + 1 {
        let mut relative_short_path_short_comps = String::new();
        let mut absolute_short_path_short_comps = String::from("/");
        let mut comp = vec!["a"; comp_len - 1].join("");
        comp.push('/');
        relative_short_path_short_comps.push_str(&comp);
        absolute_short_path_short_comps.push_str(&comp);

        path_strings.push((
            format!("Rel Short Path with {} byte comps", comp_len - 1),
            relative_short_path_short_comps,
        ));
        path_strings.push((
            format!("Abs Short Path with {} byte comps", comp_len - 1),
            absolute_short_path_short_comps,
        ));
        comp_len = comp_len * 2;
    }

    comp_len = 2;
    while comp_len <= NAME_MAX + 1 {
        let mut relative_short_path_short_comps = String::new();
        let mut absolute_short_path_short_comps = String::from("/");
        let mut comp = vec!["a"; comp_len - 1].join("");
        comp.push('/');

        for _ in 0..PATH_MAX / comp.len() {
            relative_short_path_short_comps.push_str(&comp);
            absolute_short_path_short_comps.push_str(&comp);
        }

        path_strings.push((
            format!("Rel Long Path with {} byte comps", comp_len - 1),
            relative_short_path_short_comps,
        ));
        path_strings.push((
            format!("Abs Long Path with {} byte comps", comp_len - 1),
            absolute_short_path_short_comps,
        ));
        comp_len = comp_len * 2;
    }

    // Short Paths: 1 path component
    // let mut relative_short_path_short_comps = String::new();
    // let mut absolute_short_path_short_comps = String::from("/");
    // relative_short_path_short_comps.push_str(&short_comp);
    // absolute_short_path_short_comps.push_str(&short_comp);
    // path_strings.push(("Rel Short Path Short Comp", relative_short_path_short_comps));
    // path_strings.push(("Abs Short Path Short Comp", absolute_short_path_short_comps));

    // let mut relative_short_path_long_comps = String::new();
    // let mut absolute_short_path_long_comps = String::from("/");
    // relative_short_path_long_comps.push_str(&long_comp);
    // absolute_short_path_long_comps.push_str(&long_comp);
    // path_strings.push(("Rel Short Path Long Comp", relative_short_path_long_comps));
    // path_strings.push(("Abs Short Path Long Comp", absolute_short_path_long_comps));

    // Long Paths: PATH_MAX/sizeof(comp bytes)
    // let mut relative_long_path_short_comps = String::new();
    // let mut absolute_long_path_short_comps = String::from("/");

    // for _ in 0..PATH_MAX / 2 {
    //     relative_long_path_short_comps.push_str(&short_comp);
    //     absolute_long_path_short_comps.push_str(&short_comp);
    // }

    // path_strings.push(("Rel Long Path Short Comp", relative_long_path_short_comps));
    // path_strings.push(("Abs Long Path Short Comp", absolute_long_path_short_comps));

    // let mut relative_long_path_long_comps = String::new();
    // let mut absolute_long_path_long_comps = String::from("/");

    // // +1 for separator byte
    // for _ in 0..PATH_MAX / (NAME_MAX + 1) {
    //     relative_long_path_long_comps.push_str(&long_comp);
    //     absolute_long_path_long_comps.push_str(&long_comp);
    // }

    // path_strings.push(("Rel Long Path Long Comp", relative_long_path_long_comps));
    // path_strings.push(("Abs Long Path Long Comp", absolute_long_path_long_comps));

    // // Inconsistent sized paths: Similar as long path, but randomly
    // // sized components
    // let mut relative_long_path_inconsistent_comps = String::new();
    // let mut absolute_long_path_inconsistent_comps = String::from("/");

    // let mut counter = PATH_MAX;
    // while counter > 1 {
    //     let rand = random_range(1..=cmp::min(NAME_MAX, counter));
    //     let mut a_string = String::new();

    //     for _ in 0..rand {
    //         a_string.push('a');
    //     }

    //     relative_long_path_inconsistent_comps.push_str(&a_string);
    //     absolute_long_path_inconsistent_comps.push_str(&a_string);

    //     counter -= rand;

    //     if counter > 1 {
    //         relative_long_path_inconsistent_comps.push('/');
    //         absolute_long_path_inconsistent_comps.push('/');
    //         counter -= 1;
    //     }
    // }

    // println!("Rel Long Path Inconsistent Comp chosen: {:?}", relative_long_path_inconsistent_comps);
    // println!();
    // println!("Abs Long Path Inconsistent Comp chosen: {:?}", absolute_long_path_inconsistent_comps);
    // println!();

    // Inconsistent sized component paths are generated from above randomization
    let relative_long_path_inconsistent_comps = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaa/aaa/aaaaaa/aaa".to_string();
    let absolute_long_path_inconsistent_comps = "/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/aaaaaaaaaaaa/aaa/aaaaaa/aaa".to_string();

    path_strings.push((
        "Rel Long Path Inconsistent Comp".to_string(),
        relative_long_path_inconsistent_comps,
    ));
    path_strings.push((
        "Abs Long Path Inconsistent Comp".to_string(),
        absolute_long_path_inconsistent_comps,
    ));

    for (case, path) in path_strings {
        let mut start_path_fail = path.clone();
        let mut mid_path_fail = path.clone();
        let mut end_path_fail = path.clone();
        start_path_fail.insert_str(1, "b/");
        mid_path_fail.insert_str(mid_path_fail.len() / 2, "b/");
        end_path_fail.push_str("b/");

        c.bench_function(&format!("{:?}, Components Next Rewrite", case), |b| {
            b.iter(|| black_box(components_next_iter(black_box(path.as_ref()))))
        });

        c.bench_function(&format!("{:?}, Components Next Back Rewrite", case), |b| {
            b.iter(|| black_box(components_next_back_iter(black_box(path.as_ref()))))
        });

        c.bench_function(&format!("{:?}, As Path Iter Rewrite", case), |b| {
            b.iter(|| black_box(as_path_iter(black_box(path.as_ref()))))
        });

        c.bench_function(&format!("{:?}, Components Equality Succeed", case), |b| {
            b.iter(|| black_box(eq_comps(black_box(path.as_ref()), black_box(path.as_ref()))))
        });

        c.bench_function(
            &format!("{:?}, Components Equality Fail from Start", case),
            |b| {
                b.iter(|| {
                    black_box(eq_comps(
                        black_box(path.as_ref()),
                        black_box(start_path_fail.as_ref()),
                    ))
                })
            },
        );

        c.bench_function(
            &format!("{:?}, Components Equality Fail from Mid", case),
            |b| {
                b.iter(|| {
                    black_box(eq_comps(
                        black_box(path.as_ref()),
                        black_box(mid_path_fail.as_ref()),
                    ))
                })
            },
        );

        c.bench_function(
            &format!("{:?}, Components Equality Fail from End", case),
            |b| {
                b.iter(|| {
                    black_box(eq_comps(
                        black_box(path.as_ref()),
                        black_box(end_path_fail.as_ref()),
                    ))
                })
            },
        );

        c.bench_function(&format!("{:?}, Components Comparison Succeed", case), |b| {
            b.iter(|| {
                black_box(compare_comps(
                    black_box(path.as_ref()),
                    black_box(path.as_ref()),
                ))
            })
        });

        c.bench_function(
            &format!("{:?}, Components Comparison Fail from Start", case),
            |b| {
                b.iter(|| {
                    black_box(compare_comps(
                        black_box(path.as_ref()),
                        black_box(start_path_fail.as_ref()),
                    ))
                })
            },
        );

        c.bench_function(
            &format!("{:?}, Components Comparison Fail from Mid", case),
            |b| {
                b.iter(|| {
                    black_box(compare_comps(
                        black_box(path.as_ref()),
                        black_box(mid_path_fail.as_ref()),
                    ))
                })
            },
        );

        c.bench_function(
            &format!("{:?}, Components Comparison Fail from End", case),
            |b| {
                b.iter(|| {
                    black_box(compare_comps(
                        black_box(path.as_ref()),
                        black_box(end_path_fail.as_ref()),
                    ))
                })
            },
        );

        c.bench_function(&format!("{:?}, Std Components Next", case), |b| {
            b.iter(|| black_box(std_components_next_iter(black_box(path.as_ref()))))
        });

        c.bench_function(&format!("{:?}, Std Components Next Back", case), |b| {
            b.iter(|| black_box(std_components_next_back_iter(black_box(path.as_ref()))))
        });

        c.bench_function(&format!("{:?}, Std As Path Iter", case), |b| {
            b.iter(|| black_box(std_as_path_iter(black_box(path.as_ref()))))
        });

        c.bench_function(&format!("{:?}, Std Components Equality", case), |b| {
            b.iter(|| {
                black_box(std_eq_comps(
                    black_box(path.as_ref()),
                    black_box(path.as_ref()),
                ))
            })
        });

        c.bench_function(
            &format!("{:?}, Std Components Equality Fail from Start", case),
            |b| {
                b.iter(|| {
                    black_box(std_eq_comps(
                        black_box(path.as_ref()),
                        black_box(start_path_fail.as_ref()),
                    ))
                })
            },
        );

        c.bench_function(
            &format!("{:?}, Std Components Equality Fail from Mid", case),
            |b| {
                b.iter(|| {
                    black_box(std_eq_comps(
                        black_box(path.as_ref()),
                        black_box(mid_path_fail.as_ref()),
                    ))
                })
            },
        );

        c.bench_function(
            &format!("{:?}, Std Components Equality Fail from End", case),
            |b| {
                b.iter(|| {
                    black_box(std_eq_comps(
                        black_box(path.as_ref()),
                        black_box(end_path_fail.as_ref()),
                    ))
                })
            },
        );

        c.bench_function(
            &format!("{:?}, Std Components Comparison Succeed", case),
            |b| {
                b.iter(|| {
                    black_box(std_compare_comps(
                        black_box(path.as_ref()),
                        black_box(path.as_ref()),
                    ))
                })
            },
        );
        c.bench_function(
            &format!("{:?}, Std Components Comparison Fail from Start", case),
            |b| {
                b.iter(|| {
                    black_box(std_compare_comps(
                        black_box(path.as_ref()),
                        black_box(start_path_fail.as_ref()),
                    ))
                })
            },
        );

        c.bench_function(
            &format!("{:?}, Std Components Comparison Fail from Mid", case),
            |b| {
                b.iter(|| {
                    black_box(std_compare_comps(
                        black_box(path.as_ref()),
                        black_box(mid_path_fail.as_ref()),
                    ))
                })
            },
        );

        c.bench_function(
            &format!("{:?}, Std Components Comparison Fail from End", case),
            |b| {
                b.iter(|| {
                    black_box(std_compare_comps(
                        black_box(path.as_ref()),
                        black_box(end_path_fail.as_ref()),
                    ))
                })
            },
        );
    }
}

criterion_group!(benches, bench_components_fast);
criterion_main!(benches);
