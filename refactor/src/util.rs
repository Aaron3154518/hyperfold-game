use std::path::PathBuf;

// Traits for calling except() with a String (i.e. with format!())
pub trait Catch<T> {
    fn catch(self, err: String) -> T;
}

impl<T> Catch<T> for Option<T> {
    fn catch(self, err: String) -> T {
        self.expect(err.as_str())
    }
}

impl<T, E> Catch<T> for Result<T, E>
where
    E: std::fmt::Debug,
{
    fn catch(self, err: String) -> T {
        self.expect(err.as_str())
    }
}

// Trait for getting the value of a Result regardless of Ok/Err
pub trait Get<T> {
    fn get(self) -> T;
}

impl<T> Get<T> for Result<T, T> {
    fn get(self) -> T {
        match self {
            Ok(t) | Err(t) => t,
        }
    }
}

// Trait for mapping Vec elements to strings and joining them
pub trait JoinMap<T> {
    fn map_vec<U>(&self, f: impl FnMut(&T) -> U) -> Vec<U>;

    fn join_map(&self, f: impl FnMut(&T) -> String, sep: &str) -> String {
        self.map_vec(f).join(sep)
    }
}

impl<T> JoinMap<T> for Vec<T> {
    fn map_vec<U>(&self, f: impl FnMut(&T) -> U) -> Vec<U> {
        self.iter().map(f).collect()
    }
}

impl<T, const N: usize> JoinMap<T> for [T; N] {
    fn map_vec<U>(&self, f: impl FnMut(&T) -> U) -> Vec<U> {
        self.iter().map(f).collect()
    }
}

// Trait for logic on None values in Options
pub trait NoneOr<T> {
    fn is_none_or_into(self, f: impl FnOnce(T) -> bool) -> bool;
    fn is_none_or(&self, f: impl FnOnce(&T) -> bool) -> bool;
}

impl<T> NoneOr<T> for Option<T> {
    fn is_none_or_into(self, f: impl FnOnce(T) -> bool) -> bool {
        !self.is_some_and(|t| !f(t))
    }

    fn is_none_or(&self, f: impl FnOnce(&T) -> bool) -> bool {
        match self {
            Some(t) => f(t),
            None => true,
        }
    }
}

// Trait for doing shit on tuples
pub trait SplitIter<T, U, V> {
    fn split(&self, f: impl Fn(&T, &U) -> V) -> V;
}

impl<T, U, V> SplitIter<T, U, V> for (T, U) {
    fn split(&self, f: impl Fn(&T, &U) -> V) -> V {
        f(&self.0, &self.1)
    }
}

// Splitting string into list
pub trait SplitCollect {
    fn split_collect(&self, sep: &str) -> Vec<String>;
}

impl SplitCollect for String {
    fn split_collect(&self, sep: &str) -> Vec<String> {
        self.split(sep).map(|s| s.to_string()).collect()
    }
}

impl SplitCollect for str {
    fn split_collect(&self, sep: &str) -> Vec<String> {
        self.split(sep).map(|s| s.to_string()).collect()
    }
}

#[inline]
pub fn end<T>(v: &Vec<T>, off: usize) -> usize {
    v.len().max(off) - off
}

// Manage use statements
pub fn parse_vec_path(parent_path: &Vec<String>, path: &Vec<String>) -> Vec<String> {
    let mut res_path: Vec<String> = Vec::new();
    for p in path {
        match p.as_str() {
            "super" => {
                if res_path.is_empty() {
                    res_path = parent_path[..end(parent_path, 1)].to_vec();
                } else {
                    res_path.pop();
                }
            }
            _ => res_path.push(p.to_string()),
        }
    }
    res_path
}

pub fn parse_syn_path(parent_path: &Vec<String>, path: &syn::Path) -> Vec<String> {
    parse_vec_path(
        parent_path,
        &path.segments.iter().map(|s| s.ident.to_string()).collect(),
    )
}
