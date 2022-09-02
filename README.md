# svlog

Display, filter and follow socklog log files on Void Linux.

## Features

 - Various options and filters for timestamps, priority, service and content of the logs
 - Follow changes and display new logs
 - Show logs in localtime or UTC
 - Multithreaded processing of log files

## Usage Examples

Show all logs which match the regular expression `[A-Za-z]lue.ooth` (case
insensitive) since the last boot and display timestamps in UTC.

``` sh
svlog -m "[A-Za-z]lue.ooth" -i -b --utc
```

Show all kernel logs from the previous boot with priority error or lower.

``` sh
svlog -o 1 -p ..err kernel
```

Show all kernel and daemon logs as of a certain timestamp until yesterday.

``` sh
svlog -s "2022-08-14 13:45" -u yesterday kernel daemon
```

Show the last 10 lines and all upcoming kernel logs (like `svlogtail`).

``` sh
svlog -f kernel
```

## Installation

You can find an xbps template here:
https://github.com/pascal-huber/void-templates/blob/master/srcpkgs/svlog/template.
Otherwise, `cargo` is your friend.
