# dirshare
Simple program that makes use of Git to seamlessly synchronise the contents of a directory on two or more computers.

## Why??
Why use dirshare when git exists? Dirshare is not trying to be a version control system or intended for management of complex projects. It's meant to serve a similar purpose to cloud services like OneDrive, Google Drive, iCloud, etc but free, open-source, more seamless (you don't have to upload and download from a cloud service), and not trying to harvest your data. For example, if you had a work, school, or otherwise mobile laptop and a home PC and wanted to synchronise your personal files across the two, you could either install a OneDrive client or manually upload them to and download them from your cloud storage service of choice every time you wanted to switch where you were working, or you could set up a share and have any files you place in it be seamlessly synchronised between the two computers in real time. Aditionally, for group projects that don't quite warrant a whole git repository, like school projects, you could set up a share and have every member of the group join in so that work can be done seamlessly and efficiently.

## CLI Reference

### dirshare new
Creates and configures a new share within your current directory.

#### Usage
```
dirshare new <share-name> <remote-url>
```
| Argument | Description |
|----------|-------------|
| share-name | The name of the share to create. This is also the name used to publicly identify the share. |
| remote-url | Specifies the url of the remote git repository that will be used by dirshare. |

#### Notes
- The remote repository must already exist.
- Everyone who you wish to be able to join the share must have the right permissions on your selected remote provider.

### dirshare join
Clones an existing share and configures it for synchronisation.

#### Usage
```
dirshare join <remote-url>
```
| Argument | Description |
|----------|-------------|
| remote-url | the url of the remote repository containing the share that will be cloned. |

#### Notes
- The directory is created in the current working directory.

### dirshare status
Reports the listening and synchronisation status of the share associated with the current working directory.

#### Usage
```
dirshare status
```

### dirshare start/dirshare stop
Start or pause the synchronsiation of the share associated with the current working directory.

#### Usage
```
dirshare start
```
```
dirshare stop
```
