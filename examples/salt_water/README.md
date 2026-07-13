# Salt-Water stand-alone CAPE-OPEN 1.2 Property Package

This example features a stand-alone CAPE-OPEN Property Package.

There is no configuration, and therefore persistence is not 
required nor implemented. In addition, there is no Edit dialog.

## build instructions

### Debug

```
cargo build
cobiaRegister target/debug/salt_water.dll
```

On linux, replace ```salt_water.dll``` by ```libsalt_water.so```.

### Release

```
cargo build --release
cobiaRegister target/release/salt_water.dll
```

On linux, replace ```salt_water.dll``` by ```libsalt_water.so```.

### Documentation

```
cargo doc --no-deps --document-private-items --package salt_water --open
```

## References 

* Mostafa H. Sharqawy, John H. Lienhard V, Syed M. Zubair, Desalination and Water Treatment, doi 10.5004/dwt.2010.1079
* Kishor G. Nayar, Mostafa H. Sharqawy, Leonardo D. Banchik, John H. Lienhard V, Desalination, doi 10.1016/j.desal.2016.02.024


