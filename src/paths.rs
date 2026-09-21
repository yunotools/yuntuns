use std::env;
use std::path::PathBuf;

pub fn home_dir() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("YUNTUNS_HOME").filter(|value| !value.is_empty()) {
        return Ok(PathBuf::from(path));
    }

    #[cfg(windows)]
    {
        env::var_os("LOCALAPPDATA")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .map(|path| path.join("yuntuns"))
            .ok_or_else(|| "không thể xác định thư mục plugin; hãy đặt YUNTUNS_HOME".to_owned())
    }

    #[cfg(not(windows))]
    {
        if let Some(path) = env::var_os("XDG_DATA_HOME").filter(|value| !value.is_empty()) {
            return Ok(PathBuf::from(path).join("yuntuns"));
        }

        env::var_os("HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .map(|path| path.join(".local").join("share").join("yuntuns"))
            .ok_or_else(|| "không thể xác định thư mục plugin; hãy đặt YUNTUNS_HOME".to_owned())
    }
}

pub fn bin_dir() -> Result<PathBuf, String> {
    Ok(home_dir()?.join("bin"))
}
