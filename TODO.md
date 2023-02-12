# TODO

- Implement the abilty to change the sort order of the files. This should be
  in chronological order to begin with, sorted from the datbase. 
  This will be the fallback sorting, with the ability to sort by filename
  and to reverse the sort order.
- Export
  - Copy all the selected / open files to another directory
- Modify datetime / timezone information
  - Ability to update the metadata of the images including both the time the photo was taken
    along with the timezone associated with it.
  - Also update the location of the file based on the new datetime information
  - This would probably require storing the local time, along with the timezone
- View multiple directories
  - This would involve the use of multiple selection for the directories and then
    handling this in the directory imports
- Include Tags
- Full text searching (sqlite MATCH)
- configure size of thumbnails
- Implement a texture cache with better control
  This moves the control of the caching from the picture objects to the application
  Should use a LRU cache, which will make going back and forth between images
  a quick process
- Bind the values stored within the application to the UI. 
  That is the directory tree and the pictures within the directory.
  These should be updated by updating the state, that is
  querying the database.
- Improve fileters
  - Could use a toggle button for each of the values of the enums, allowing the filter to 
    match any combination of values.
- Implement Hidden files
  - Start with defaulting to hidden

## Ideas

- Facial Recognition
- Item recognition

## DONE
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
