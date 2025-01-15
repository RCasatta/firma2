At the moment there is no cross-complation support and it must be built on an arm machine.

```
nix build .#image.rpi4
```


## Disk space

On machine with small temp space it may required to use:

```
export TMPDIR=/mnt/disk-with-space/tmp
```

To recover space from the nix store:

```
nix-collect-garbage
```
