# Interconnects

OHX stores interconnects between Addons in this directory as json files.
Files are watched for changes and re-loaded if necessary.

Files with the extension `.json.new` will be validated first and only
applied if the validation succeeded and just removed otherwise. 
