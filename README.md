# Edit-ini [![GithubAction_badge]][GithubAction_link]

Command line tool for editing .ini files

[GithubAction_badge]: https://github.com/Chocobo1/edit-ini/workflows/CI/badge.svg
[GithubAction_link]: https://github.com/Chocobo1/edit-ini/actions

## Usage
```
Usage: edit-ini [OPTIONS]

Options:
  -i, --input <file>                  Input file to read from. Use `-` to read from stdin.
  -s, --set <Section> <Key=Value>     Set a section/key-value pair. Can be invoked multiple times.
  -r, --remove <Section> <Key=Value>  Remove a section/key-value pair. Can be invoked multiple times.
  -h, --help                          Print help information (use `--help` for more detail)
  -V, --version                       Print version information
```

## Examples

* Given the following `app.ini` file:
  ```ini
  [Animal]
  Cat=meow
  ```

  * To add a key-value pair `Dog=bark` to the file, run:
    ```shell
    edit-ini -s Animal Dog=bark -i app.ini -o app.ini
    ```

  * To add a key-value pair `Dog=bark` and show it on screen (via stdout):
    ```shell
    edit-ini -s Animal Dog=bark -i app.ini
    ```

* Create a new `clearance.ini` file with some key-value pairs:
  ```shell
  edit-ini \
    -s "Top Secret" Name=Alfa \
    -s Secret Name=Bravo \
    -s Confidential Name=Charlie \
    -o clearance.ini
  ```
  Gives:
  ```ini
  [Top Secret]
  Name=Alfa

  [Secret]
  Name=Bravo

  [Confidential]
  Name=Charlie
  ```

## License
See [LICENSE](./LICENSE) file
