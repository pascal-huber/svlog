# svlog

Display, filter and follow socklog log files on Void Linux.

## Usage Examples

Show all logs which contain "blue" (case insensitive) since the last boot and
display timestamps in UTC.

``` sh
svlog kernel -b -m blue -i --utc 
```

Show all kernel logs from the previous boot with priority error or lower.

``` sh
svlog -o 1 -p ..err kernel
```

Show all kernel logs as of a certain timestamp until yesterday.

``` sh
svlog -s "2022-08-14 13:45" -u yesterday kernel
```

Show the last 10 lines and all upcoming logs from services kernel and xbps (like svlogtail).

``` sh
svlog -f xbps kernel
```


