//! 本地配置信息加载库(本地配置文件由环境变量 `DEFAULT_GLOBAL_CONFIG` 指定)
//!
//! ```no_run
//! // 初始化并获取全局配置
//! let settings = global_config().get().unwrap();
//! // 获取配置中的某项配置，返回值为字符串
//! let name = settings.get_string("delist.name").unwrap();
//! // 获取配置中的某项配置，返回值为路径PathBuf
//! let db_file = settings.get_path("delist.db_file").unwrap();
//! ```

use anyhow::{bail, Context};
use config::Config;
use std::{
    ops::Deref,
    path::{Path, PathBuf},
    sync::OnceLock,
};

/// 全局配置(线程安全)
static CELL: OnceLock<Settings> = OnceLock::new();

/// 加载全局默认配置文件(配置文件由环境变量 `DEFAULT_GLOBAL_CONFIG` 指定)
///
/// 全局配置线程安全
pub fn global_config() -> &'static OnceLock<Settings> {
    CELL.get_or_init(|| {
        Settings::new(None).unwrap_or_else(|e| panic!("load global config fail: {e}"))
    });
    &CELL
}

/// 从指定的配置文件中加载配置
///
/// ```no_run
/// // 参数为None，表示根据 DEFAULT_GLOBAL_CONFIG 环境变量指定的默认全局配置文件路径来加载配置
/// // 否则应通过参数设置路径`Some(<CONFIG_FILE>)`
/// let settings = Settings::new(None);
/// // 获取 delist.name 属性的值
/// let config_value = settings.get_string("delist.name");
/// ```
#[derive(Debug)]
pub struct Settings {
    /// 配置文件所在目录(亦是配置文件的默认相对路径)
    pub config_dir: PathBuf,
    /// 配置文件的文件名
    pub config_filename: String,
    settings: Config,
}

impl Deref for Settings {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.settings
    }
}

impl Settings {
    /// 加载配置文件
    ///
    /// 如果不指定参数，则根据环境变量`DEFAULT_GLOBAL_CONFIG`的值加载默认配置文件，
    /// 指定参数，表示加载指定的配置文件
    pub fn new(config_file: Option<&str>) -> anyhow::Result<Self> {
        let mut settings = Config::builder();

        let path = match config_file {
            Some(f) => Path::new(f).to_path_buf(),
            None => {
                let default_config = std::env::var("DEFAULT_GLOBAL_CONFIG")
                    .context("Environment Variable `DEFAULT_GLOBAL_CONFIG` empty or not defined")?;
                Path::new(&default_config).to_path_buf()
            }
        };
        if !path.exists() {
            bail!("config file not found: {}", path.to_string_lossy());
        }

        settings = settings.add_source(config::File::with_name(path.to_str().unwrap()));

        Ok(Settings {
            config_dir: PathBuf::from(path.canonicalize()?.parent().unwrap()),
            config_filename: path.file_name().unwrap().to_string_lossy().to_string(),
            settings: settings.build()?,
        })
    }

    /// 全局配置文件所在目录(即当前配置文件所在目录)
    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    // 是对 Config 的`get_xx`方法的补充
    /// 获取配置中的路径。
    ///
    /// - 如果key不存在或者获取的值为空字符串，则返回Err
    /// - 如果获取的值value是绝对路径(例如`/path/to/file`)，则返回Ok(value)对应的路径
    /// - 如果获取的值value是相对路径(例如`./path/file`)，则相对于当前全局配置文件所在的目录，
    ///   并返回附加子路径后的完整绝对路径
    pub fn get_path(&self, key: &str) -> Result<PathBuf, config::ConfigError> {
        let value = self.get_string(key)?;
        if value.is_empty() {
            Err(config::ConfigError::Message("empty value".into()))
        } else {
            let p = Path::new(&value);
            if p.is_absolute() {
                Ok(p.to_path_buf())
            } else {
                Ok(self.config_dir().join(&value))
            }
        }
    }
}
