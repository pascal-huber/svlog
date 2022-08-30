# svlog

Display, filter and follow socklog log files on Void Linux.

## Usage Examples

Show all logs which contain "blue" (case insensitive) and were logged since the
last boot. 

``` sh
svlog kernel -b -m blue
```

Show all kernel logs from the previous boot with priority error or lower.

``` sh
svlog -p ..err -o 1 kernel
```

You want to see all kernel logs since a certain timestamp until yesterday.

``` sh
svlog -s "2022-08-14 13:45" -u yesterday kernel
```

Show the last 10 lines and all upcoming logs from services kernel and xbps.

``` sh
svlog -n xbps kernel
```


