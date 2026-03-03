use std::borrow::Cow;

pub fn is_safe(path: &str) -> bool {
    for seg in path.split('/') {
        if seg.starts_with("..") {
            return false;
        } else if seg.contains('\\') {
            return false;
        } else if cfg!(windows) && seg.contains(':') {
            return false;
        }
    }
    true
}

pub fn extension(name: &str) -> &str {
    name.rsplit_once(".").map_or_else(|| "", |(_, v)| v)
}

/// 规范化路径分隔符：将所有反斜杠 `\` 替换为正斜杠 `/`
pub fn normalize_sep(path: &str) -> Cow<'_, str> {
    if path.contains('\\') {
        path.replace('\\', "/").into()
    } else {
        path.into()
    }
}

#[derive(Debug, Clone)]
pub struct PathJoin {
    root: String,
}

impl PathJoin {
    pub fn root(root: impl Into<String>) -> Self {
        Self { root: root.into() }
    }

    pub fn join<T: AsRef<str>>(mut self, src: T) -> Self {
        let src = PathJoin::trim_start_sep(src.as_ref());

        if !self.root.is_empty() && !PathJoin::ends_with_sep(&self.root) {
            self.root.push('/');
        }

        self.root.push_str(src);
        self
    }

    pub fn into_string(self) -> String {
        self.root
    }

    fn trim_start_sep(path: &str) -> &str {
        path.trim_start_matches(|c| c == '/' || c == '\\')
    }

    fn ends_with_sep(path: &str) -> bool {
        path.ends_with('/') || path.ends_with('\\')
    }
}

impl std::fmt::Display for PathJoin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.root)
    }
}
