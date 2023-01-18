# TODO

- Implement importing of files
  Just the quick and dirty method to get started with
  - Bind to button / filechooser
  - create function adding files to database
- Implement filtering of files based on criteria
- configure size of thumbnails
- Implement a texture cache with better control
  This moves the control of the caching from the picture objects to the application
  Should use a LRU cache, which will make going back and forth between images
  a quick process
- Bind the values stored within the application to the UI. 
  That is the directory tree and the pictures within the directory.
  These should be updated by updating the state, that is
  querying the database.
- Implement the abilty to change the sort order of the files. This should be
  in chronological order to begin with, sorted from the datbase. 
  This will be the fallback sorting, with the ability to sort by filename
  and to reverse the sort order.

## DONE
- Sort the pictures from the database.
- Update then UI when a new directory is added
- handle scaling factor for hdpi screens
- Implement the ability to pick / reject a picture
- implement keybindings for selection
