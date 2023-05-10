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
    fn join_map<F>(&self, f: F, sep: &str) -> String
    where
        F: Fn(&T) -> String;
}

impl<T> JoinMap<T> for Vec<T> {
    fn join_map<F>(&self, f: F, sep: &str) -> String
    where
        F: Fn(&T) -> String,
    {
        self.iter().map(f).collect::<Vec<_>>().join(sep)
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
