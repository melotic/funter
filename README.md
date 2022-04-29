<img align="left" width="64" height="64" src="https://i.imgur.com/7B6zGax.png" alt="logo">

# funter

A fast, multithreaded tool for finding regex matches in text & binary files. This tool is built for A&D CTFs.

It can help you find flags on a given box without searching endlessly.




## Features

- Searches through binary files & text files
- Multithreaded
- ~1MB footprint
- Cross platform


## Usage/Examples

```bash
funter "htb{[A-Za-z0-9]{32}}" /home/justin
funter "htb{[A-Za-z0-9]{32}}" # searches /
```

