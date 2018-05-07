# oida - painless SDK debugging
This project was started for only one reason: to make debugging after-the-fact production issues as painless as possible. Since most of the time the only trail left behind is a log, the binary analyzes the logs and creates an index of events which can then be visualized in various ways.

As we add features, expect some more documentation here.

And if you want to know where the name came from, check out this [youtube video](https://www.youtube.com/watch?v=iuXR53ex4iI) explaining the name and then just figure out my usual first reaction to troubleshooting tickets.

# Installation
For now we are not shipping binaries (lol, what did you expect?) so you have to build it for yourself. Its fairly easy with rust:

 - If you haven't already, install [rust](https://rustup.rs/).
 - Clone the project `git clone https://github.com/daschl/oida.git`
 - Change into the directory and run `cargo build --release`

The binary can now be found under `target/release/oida`.

# Usage
oida is split into two commands. One to analyze your log file and to create an index of interesting events happening. And the other command to then visualize the index information. For now only CLI output is supported, but we want to add a proper web UI as well.

## Analyze a Logfile
You can always call `-h` to get help on the commands:

```
$ oida -h
oida 0.1
Michael Nitschinger <michael@nitschinger.at>
Troubleshoot the Java SDK, oida.

USAGE:
    oida <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    check    analyze a logfile and generate a meta index for later use.
    help     Prints this message or the help of the given subcommand(s)
    show     visualize the details of the index file
```

Both commands need a config file via the `-c` argument. A sample can be found [here](https://github.com/daschl/oida/blob/master/example_config.toml). If you name it `oida.toml` then its picked up automatically and used for all the commands.

If we start with the following config file

```toml
[check]
input = "tmp/input.log"
pattern = "^%{TIMESTAMP_ISO8601:timestamp} %{NUMBER:ign} \\| %{LOGLEVEL:level}%{SPACE} \\| %{GREEDYDATA:message}$"

[show]
```

```
$ oida check
> Starting to analyze "tmp/input.log"
1.36 MB / 1.36 MB [--------------------------------------------------------------------------------------------------------------------------------------------] 100.00 % 1.71 MB/s  > Completed
> Dumping Index into File "index.oida"
> Completed
```

Once the index file is created, you can visualize it via the CLI for now:

```
$ oida show
> Loading Index from File "index.oida"
> Completed
> Printing Stats to CLI

Topology Events
---------------

  16:16:46 node + 10.80.161.181
  16:16:46 node + 10.80.162.14
  16:16:46 node + 10.80.162.239
  16:16:48 node + 10.80.161.133
  16:16:48 node + 10.80.161.136
  16:16:48 node + 10.80.161.144
  16:16:48 node + 10.80.161.146
  16:16:48 node + 10.80.161.150
  16:16:48 node + 10.80.161.184
...
```
