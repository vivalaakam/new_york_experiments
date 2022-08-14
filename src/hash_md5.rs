pub fn hash_md5<S1>(str: S1) -> String
where
    S1: Into<String>,
{
    format!("{:x}", md5::compute(str.into().as_bytes()))
}
