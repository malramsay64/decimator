# TODO

- Select multiple items
- Include Tags
- Full text searching (sqlite MATCH)
- Modal to confirm replacing of all thumbnails
- Modal to check updating directory path if not found
- Include rating scale
- Go to preview from Grid on double click
- progress bar
- Hover labels for buttons (tooltips)
- Run migrations and database on app initialisation
- log and ignore empty / malformed files on import
- Documentation of code / modules / functions / classes
- Update file location
  - Instead of crashing when directory is not found, prompt for dialog to find again
  - Update all images, not just those shown (that is, the hidden ones)
- Modify datetime / timezone information
  - Ability to update the metadata of the images including both the time the photo was taken
    along with the timezone associated with it.
  - Also update the location of the file based on the new datetime information
  - This would probably require storing the local time, along with the timezone
- configure size of thumbnails
  - Within the application settings
  - This should be the value also for HiDPI screens
  - Prompt noting that HiDPI should be double

## Ideas

- Computing image transformations using WGPU compute

- Facial Recognition
- Item recognition

## DONE
- Show selected item in directory list
- Style
  - Add bars between buttons
  - Shrink buttons for Picked
  - alignment of buttons for picked
- Order directories
- Scroll to selected widget
- Zoom in based on max not min
- Style of Widgets
- Implement Grid View
- Double click to zoom
- Implement the ability to change the sort order of the files. This should be
  in chronological order to begin with, sorted from the database. 
  This will be the fallback sorting, with the ability to sort by filename
  and to reverse the sort order.
- Perform the conversion from an image to a pixbuf when binding to a widget
- Zoom in on preview image
  - This will require storing both the original image and the zoomed paintable
- Create a grid view, allowing the export of a selection
- Export
  - Copy all the selected / open files to another directory
- Implement filtering of files based on criteria (pick / unpick)
- Import RAW files along with JPEGs
- Sort the pictures from the database.
- Update then UI when a new directory is added
- handle scaling factor for hdpi screens
- Implement the ability to pick / reject a picture
- implement keybindings for selection
- Implement importing of files
  Just the quick and dirty method to get started with
  - Bind to button / filechooser
  - create function adding files to database
- View multiple directories
  - This would involve the use of multiple selection for the directories and then
    handling this in the directory imports
- Improve fileters
  - Could use a toggle button for each of the values of the enums, allowing the filter to 
    match any combination of values.
- Implement Hidden files
  - Start with defaulting to hidden
- Bind the values stored within the application to the UI. 
  That is the directory tree and the pictures within the directory.
  These should be updated by updating the state, that is
  querying the database.
- Implement a texture cache with better control
  This moves the control of the caching from the picture objects to the application
  Should use a LRU cache, which will make going back and forth between images
  a quick process
