# svlog

Display, filter and follow socklog log files on Void Linux.

## Example use cases

There is something wrong with your bluetooth and you want to see all logs which
contain "blue" (case insensitive) since you booted the machine:

``` sh
svlog kernel -b -m blue
```

Your PC froze and you had to reset it. You have no idea why this happened and
want to see all kernel logs from the previous boot.

``` sh
svlog -o 1 kernel
```

You want to see all upcoming kernel and xbps logs (for whatever reason).

``` sh
svlog -n xbps kernel
```
