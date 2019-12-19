# Naming

* A *_registry is populated during runtime and not disk serialized.
* A *_store is serialized to disk on a one-item-one-file strategy and read again on start up.
  A store usually watches the target directory for changes and re-loads files and applies techniques
  to not load files that have been serialized moments ago. 