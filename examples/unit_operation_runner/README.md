# CAPE-OPEN Stand-Alone Unit Operation Runner

This example allows for selecting a CAPE-OPEN Unit 
Operation in combination with a CAPE-OPEN Property 
Package. 

The GUI is Tauri v2 based. 

Once the specification is complete and each time the 
specification is modified, the Unit Operation will 
automatically be calculated and the results appear
in the window.

The content can be saved in xml format. The xml 
contains all  input and output as well as the private 
data of the Unit Operation and Property Package. 

One can specify the saved file on the command line to 
automatically open the file.

```
unitoperationrunner <input-file-name>
```


On can manipulate the content of the xml to modify 
stream conditions or paramater values, and re-run
the file using 

```
unitoperationrunner --run <input-file-name>
```

This will entirely bypass the GUI and can be used 
in automation tasks and unit testing tasks.


## build instructions

### Prerequisites

Please install the Tauri build tools:

```
cargo install tauri-cli --version "^2.0.0" --locked
```

### Debug

```
cargo tauri build --debug
```

### Release

```
cargo tauri build
```

### Documentation

```
cd src-tauri
cargo doc --no-deps --document-private-items --open
```
