# WebLauncher - execute commands through a web interface

## Important
WebLauncher has not been designed with any kind of security in mind. You should not use this. If you use this anyways, **do not expose it to a public network**.

## What is does
WebLauncher is meant to be a way of controlling devices remotely with user configurable commands.

## How to use
* Create a config file either in `${XDG_CONFIG_HOME}/weblauncher/config.toml` or at any arbitrary location (in which case you will need to use the `configuration` flag). The file uses the [TOML syntax](https://toml.io/en/), and you can see examples in the `tests` directory.

* Run the program.

* Access the dashboard by opening the URL output in your browser, or make your own requests directly.
