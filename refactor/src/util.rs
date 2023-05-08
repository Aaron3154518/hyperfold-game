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

#[inline]
pub fn end<T>(v: &Vec<T>, off: usize) -> usize {
    v.len().max(off) - off
}
