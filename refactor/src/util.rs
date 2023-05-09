pub trait Expect<T> {
    fn catch(self, err: String) -> T;
}

impl<T> Expect<T> for Option<T> {
    fn catch(self, err: String) -> T {
        self.expect(err.as_str())
    }
}

impl<T, E> Expect<T> for Result<T, E>
where
    E: std::fmt::Debug,
{
    fn catch(self, err: String) -> T {
        self.expect(err.as_str())
    }
}

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

#[inline]
pub fn end<T>(v: &Vec<T>, off: usize) -> usize {
    v.len().max(off) - off
}

// Manage use statements
pub fn parse_use(parent_path: &Vec<String>, use_path: &Vec<String>) -> Vec<String> {
    let mut path = Vec::new();
    for p in use_path {
        match p.as_str() {
            "super" => {
                if path.is_empty() {
                    path = parent_path[..end(parent_path, 1)].to_vec();
                } else {
                    path.pop();
                }
            }
            _ => path.push(p.to_string()),
        }
    }
    path
}
