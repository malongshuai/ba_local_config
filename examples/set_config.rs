use local_config::{global_config, Settings};

fn main() {
    // 从指定的配置文件中加载配置
    {
        let config = Settings::new(Some("./examples/test_global_settings.toml")).unwrap();
        println!("{:?}", config);
        println!("{:?}", config.get_string("file.name"));

        let file = config.get_path("delist.delist_db_file").unwrap();
        println!("file path: {:?}", file);
    }

    // 全局初始化加载配置文件，从环境变量`DEFAULT_GLOBAL_CONFIG`指定的配置文件中读取配置并加载
    {
        let config1 = global_config();
        let x = config1.get().unwrap();
        println!("{:?}", x.get_string("delist.delist_db_file"));
    }
}
