# 🛠 tool-tool

tool-tool - a light-weight meta-tool to version and install tool dependencies for your software projects

[![Crates.io](https://img.shields.io/crates/v/tool-tool.svg)](https://crates.io/crates/tool-tool)
[![Actions Status](https://github.com/tool-tool-team/tool-tool/workflows/Continuous%20integration/badge.svg)](https://github.com/tool-tool-team/tool-tool/actions?query=workflow%3A%22Continuous+integration%22)
[![coverage report](https://gitlab.com/manuelwoelker/tool-tool/badges/master/coverage.svg)](https://gitlab.com/manuelwoelker/tool-tool/-/commits/master)

**WARNING**: Work in progress - comments, contributions and feedback welcome

tool-tool is simple way to automatically manage a project's tool dependencies. This ensures that all contributers and CI systems are
using the right tools with the proper version, without having to hunt them down manually.

You should never have to worry about:

 * Which version of the compiler do I need to use in this branch?
 * What JDK is necessary for this build?
 * Where do I get the correct version of node/yarn/maven/etc...?
 * Why do I get weird errors when compiling old branches with new tool versions?
 
 ## How it works:
 
 1. A configuration file (`.tool-tool.v1.yaml`) in the project repository root defines all the tool dependencies and where to download them.
 2. Small bootstrap binaries (`tt`, `tt.exe`) for all development platforms are also checked into your repository.
 3. All tool calls are then made through this bootstrap program. It parses the configuration file, downloads and caches the tools and executes the given command
 
A sample yarn invocation:
 
 ```
tt yarn install
```

Sample `.tool-tool.v1.yaml`

```
tools:
    - name: lsd
      version: 0.17.0
      download:
        linux: https://github.com/Peltoche/lsd/releases/download/0.17.0/lsd-0.17.0-x86_64-unknown-linux-gnu.tar.gz
        windows: https://github.com/Peltoche/lsd/releases/download/0.17.0/lsd-0.17.0-x86_64-pc-windows-msvc.zip
```
