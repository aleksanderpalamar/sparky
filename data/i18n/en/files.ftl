menu-open = Open
menu-open-with = Open with...
menu-cut = Cut
menu-copy = Copy
menu-move-to = Move to...
menu-copy-to = Copy to...
menu-rename = Rename...
menu-compress = Compress...
menu-extract = Extract Here
menu-email = Email...
menu-trash = Move to Trash
menu-delete = Delete Permanently
menu-properties = Properties
menu-new-folder = New folder...
menu-new-file = New document...
menu-paste = Paste
menu-select-all = Select all
menu-restore = Restore
menu-restore-all = Restore all
trash = Trash
window-title = { $location } — Files
window-title-trash = Trash — Files
copy-suffix = copy
copy-number = copy { $number }
trash-invalid-path = invalid path in trash record
trash-invalid-name = invalid name in trash
trash-invalid-directory = invalid trash directory
trash-unavailable = this location has no trash
application-unavailable = application unavailable
item-not-found = item not found
trash-ambiguous-name = ambiguous trash name
archive-different-sources = different source directories
archive-invalid-source = invalid source name
archive-invalid-target = invalid target
archive-failed = could not compress items
grid = Grid
list = List
grid-hint = Grid · Ctrl+1
list-hint = List · Ctrl+2
restore-item = Restore item
empty-trash = Empty Trash
filter-tag = filter: { $filter }
hidden-visible = hidden files visible
places = Places
devices = Devices
no-volumes = No volumes available
device-query-failed = device query: { $error }
device-departed = The device is no longer mounted
volume-of = { $size } volume
not-mounted = Not mounted
mounted = mounted
state-not-mounted = not mounted
mounting = Mounting…
unmounting = Unmounting…
unmount-hint = Unmount · Alt+U for the open volume
eject-hint = Safely eject physical drive and all its partitions
ejecting = Ejecting…
volume-hint = { $path } · { $filesystem }
    { $mountpoint }
    Alt+{ $index }: mount and open
folder = Folder
file = File
unknown = Not available
name = Name
type = Type
path = Path
size = Size
modified = Modified
permissions = Permissions
open-file-with = Open { $file } with...
cancel = Cancel
filter = Filter
filter-hint = Enter applies · Esc clears
rename = Rename
confirm-hint = Enter confirms · Esc cancels
create-hint = Create (end with / for a folder)
create-keys = Enter creates · Esc cancels
new-folder = New folder
new-file = New document
destination-hint = Folder path · Enter confirms · Esc cancels
compress = Compress (.tar.gz)
confirm-empty-trash = Permanently delete { $count } items from Trash? Enter confirms · Esc cancels
move-to = Move to
copy-to = Copy to
confirm-trash-one = Move { $name } to Trash? Enter confirms · Esc cancels
confirm-trash-many = Move { $count } items to Trash? Enter confirms · Esc cancels
confirm-delete-one = Delete { $name } permanently? This cannot be undone. Enter confirms · Esc cancels
confirm-delete-many = Delete { $count } items permanently? This cannot be undone. Enter confirms · Esc cancels
short-hints = Ctrl+1 grid · Ctrl+2 list
browse-hints = Enter opens · h goes back · / filters · Ctrl+1 grid · Ctrl+2 list
one-item = 1 item
many-items = { $count } items
one-marked = 1 selected
many-marked = { $count } selected
one-cut = 1 cut
many-cut = { $count } cut
one-copied = 1 copied
many-copied = { $count } copied
place-home = Home
place-recent = Recent
place-favorites = Favorites
place-network = Network
network-browse = Browse network
network-loading = Opening network location…
favorite-added = Added to favorites
favorite-removed = Removed from favorites
favorite-action = Toggle favorite
favorite-remove = Remove
sidebar-empty = No items available
place-desktop = Desktop
place-documents = Documents
place-downloads = Downloads
place-pictures = Pictures
place-music = Music
place-videos = Videos
need-destination = Choose a destination folder
destination-not-folder = The destination is not a folder
restored-to = Restored to { $path }
nothing-to-paste = Nothing to paste
new-document = New document
renamed-to = Renamed to { $name }
archive-busy = A compression is already running
archive-default-name = files
archive-extension = Use a file name ending in .tar.gz
already-exists = An item named { $name } already exists
compressing = Compressing { $count }...
archive-created = Archive created: { $path }
image-copied = Image copied to the clipboard
empty-trash-state = Trash is empty
emptying-trash = Emptying trash...
no-app = No application available for this file type
compress-before-mail = Compress the folders before sending them by email
device-not-mounted = the device is not mounted: { $device }
invalid-name = invalid name
item-exists = an item named { $name } already exists
into-itself = cannot paste { $name } into itself
no-visible-items = No visible items
empty-folder = Empty folder
conflict-question = { $count ->
    [one] { $name } already exists at the destination
   *[other] { $count } items already exist at the destination
  } · Enter or m keeps both · s replaces · p skips · Esc cancels
job-copying = Copying { $done } of { $total } · { $percent }% · Esc cancels
job-moving = Moving { $done } of { $total } · { $percent }% · Esc cancels
job-busy = Wait for the current operation to finish
job-cancelling = Cancelling…
job-cancelled = { $count ->
    [0] Cancelled before the first item
    [one] Cancelled after 1 item
   *[other] Cancelled after { $count } items
  }
job-copied = { $count ->
    [0] No items copied
    [one] 1 item copied
   *[other] { $count } items copied
  }
job-moved = { $count ->
    [0] No items moved
    [one] 1 item moved
   *[other] { $count } items moved
  }
job-skipped = { $count ->
    [one] 1 skipped
   *[other] { $count } skipped
  }
undone = Action undone
nothing-to-undo = Nothing to undo
undo-failed = Could not undo: { $error }
trash-of = Trash · { $disk }
clipboard-not-published = Copied in Sparky Files, but the system clipboard was not updated: { $error }
deleted-items = { $count ->
    [0] No items deleted
    [one] 1 item deleted
   *[other] { $count } items deleted
}
sort-name = name
sort-size = size
sort-modified = modification date
sort-type = type
sorted-by = Sorted by { $key } { $arrow }
extract-failed = could not extract the archive
job-extracting = Extracting { $done } of { $total } · { $percent }% · Esc cancels
extracted-into = Extracted into { $name }
extracted-many = { $count ->
    [0] No archives extracted
    [one] 1 archive extracted
   *[other] { $count } archives extracted
  }
batch-rename = Rename { $count } items
batch-hint = {"{"}n{"}"} numbers · old -> new replaces a part
batch-invalid = Invalid name: { $name }
batch-repeated = More than one item would be named { $name }
batch-taken = An item named { $name } already exists
batch-unchanged = No name would change
batch-renamed = { $count ->
    [0] No items renamed
    [one] 1 item renamed
   *[other] { $count } items renamed
  }
search = Search subfolders
search-keys = Enter searches · Esc cancels
search-title = Search for “{ $query }” in { $place }
search-running = { $count ->
    [0] Searching… · Esc goes back
    [one] Searching… 1 found · Esc goes back
   *[other] Searching… { $count } found · Esc goes back
  }
search-complete = { $count ->
    [0] No results · Esc goes back
    [one] 1 result · Esc goes back
   *[other] { $count } results · Esc goes back
  }
search-limited = { $count ->
    [0] No results · Esc goes back
    [one] Showing the first result · Esc goes back
   *[other] Showing the first { $count } results · Esc goes back
  }
tab-search = Search: { $query }
