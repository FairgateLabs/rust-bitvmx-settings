# Bitvmx Settings


This project is a library for loading and managing configuration files, supporting both default and custom configurations through environment variables and file paths.

## ⚠️ Disclaimer

This library is currently under development and may not be fully stable.
It is not production-ready, has not been audited, and future updates may introduce breaking changes without preserving backward compatibility.

## Usage

1. **Load Default Configuration**:
   Use the `load` method to load the default configuration based on the `BITVMX_ENV` environment variable or the default development configuration.

   ```rust
   let config: Config = settings::load<T>()?;
   ```

2. **Load Configuration from a Specific File**:
   If you want to load a configuration from a specific file, use the `load_config_file` method. You can pass an `Option<String>` with the file name.

   ```rust
   let config_file = Some("custom_config.yaml".to_string());
   let config: Config = settings::load_config_file(config_file)?;
   ```

### Configuration File

The configuration files are expected to be in YAML format and located in the `config` directory. The default configuration file is `development.yaml`.

### Environment Variable

Set the `BITVMX_ENV` environment variable to specify which configuration file to use. For example, setting `BITVMX_ENV=production` will load `config/production.yaml`.

### Command-line Arguments

You can also specify the configuration file directly using the `--configuration` or `-c` flag:

## Contributing
Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## License
This project is licensed under the MIT License.

