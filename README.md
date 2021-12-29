# svlog

Display, filter and follow socklog log files on Void Linux.

## Example usage

Show kernel logs since last boot containing the "bluetooth"

``` sh
svlog kernel -b -m blue
```

Show upcoming xbps and kernel logs.

``` sh
svlog -n -f xbps kernel
```
