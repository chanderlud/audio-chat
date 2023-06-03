from os import path, getenv, getcwd, makedirs, remove
from pathlib import Path
from appdirs import user_data_dir


# check if we have permission to write to a directory
def can_write_to_dir(directory: str) -> bool:
    test_file_path = path.join(directory, "test_file")  # create a test file path

    try:
        # try to write to the test file
        with open(test_file_path, "w") as test_file:
            test_file.write("test")

        # if we get here, we were able to write the file
        remove(test_file_path)  # clean up
        return True
    except IOError:
        # if we couldn't write the file, we don't have permission
        return False


APPDATA = getenv("APPDATA").replace("Roaming", "Local") + "\\Programs\\Audio Chat"  # format appdata path
ACCESS = can_write_to_dir(getcwd())  # check if the installation location is writeable

# only create appdata folder if it doesn't exist and the installation location is not writeable
if not path.exists(APPDATA) and not ACCESS:
    makedirs(APPDATA)


# only used if the installation location is not writeable
def to_appdata(file_name: str) -> str:
    return path.join(APPDATA, file_name)


# format file paths
def cv(file_name: str) -> str:
    if not ACCESS:
        return to_appdata(file_name)  # if the installation location is not writeable, use appdata
    else:
        return file_name  # otherwise, use the installation location


def download_path() -> str:
    # format downloads path
    downloads_path = path.join(path.expanduser("~"), "Downloads")

    # create downloads folder if it doesn't exist
    if not path.exists(downloads_path):
        makedirs(downloads_path)

    return downloads_path
