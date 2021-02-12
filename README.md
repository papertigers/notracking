# notracking

`notracking` is a utility to download and validate [notracking] information for
[dnsmasq]. The utility takes an optional command, which is useful for things
like restarting/refreshing the `dnsmasq` daemon itself.

[notracking]: https://github.com/notracking/hosts-blocklists
[dnsmasq]: http://www.thekelleys.org.uk/dnsmasq/doc.html


### Help

```
link - rustdev ~/src/notracking (git:master) $ notracking -h
Usage: notracking [options] [<command>...]

Options:
    -h, --help          Print this help message
    -d DIRECTORY        Directory to store notracking files in
```

### Example

```
root@dns:~# ./notracking -d /opt/local/etc/dnsmasq/ svcadm -v restart dnsmasq
Feb 12 02:11:25.563 INFO getting domains at https://raw.githubusercontent.com/notracking/hosts-blocklists/master/domains.txt
Feb 12 02:11:28.923 INFO installed domains to "/opt/local/etc/dnsmasq/domains.txt"
Feb 12 02:11:28.923 INFO getting hostnames at https://raw.githubusercontent.com/notracking/hosts-blocklists/master/hostnames.txt
Feb 12 02:11:29.879 INFO installed hostnames to "/opt/local/etc/dnsmasq/hostnames.txt"
Feb 12 02:11:29.879 INFO exec ["svcadm", "-v", "restart", "dnsmasq"]
Feb 12 02:11:29.915 INFO O| Action restart set for svc:/pkgsrc/dnsmasq:default.
```
