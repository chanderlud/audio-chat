from os import path
from random import randint
from threading import Thread, current_thread
from time import sleep

from yaml import load, dump, Loader

from _nuitka import cv


class Config(object):
    DEFAULT = {
        "controlPort": randint(49152, 65535),
        "audioPorts": "20000-21000",
        "inputSensitivity": -16.0,
        "inputDevice": 0,
        "outputDevice": 0,
        "outputVolume": 0.0,
        "inputVolume": 0.0,
        "debugLevel": 30,
        "trayApp": True,
        "useRnnoise": False,
        "VADThreshold": 0.6,
        "VADGracePeriod": 0.4,
        "retroactiveVADGracePeriod": 0.0
    }

    controlPort: int
    audioPorts: str
    inputSensitivity: float
    inputDevice: str
    outputDevice: str
    outputVolume: float
    inputVolume: float
    debugLevel: int
    trayApp: bool
    useRnnoise: bool
    VADThreshold: float
    VADGracePeriod: float
    retroactiveVADGracePeriod: float
    thread: Thread

    def __init__(self):
        if not path.exists(cv("config.yml")):
            self.default()
        else:
            self._load()

        self.thread = Thread(target=self._update)
        self.thread.start()

        # config is outdated version, update while retaining data
        if self.__dict__.keys() != self.DEFAULT.keys():
            old_data = dict(self.__dict__)  # copy the original values

            self.default()  # restore config to default

            # restore old options
            for (key, value) in old_data.items():
                if self.__dict__.get(key) != value:
                    self.__dict__[key] = value

            self._save()

    # create default config
    def default(self):
        with open(cv("config.yml"), "w+") as f:
            dump(self.DEFAULT, f)

        self._load()

    # load config from disk
    def _load(self):
        with open(cv("config.yml"), "r") as f:
            config_dict = load(f, Loader)
            self.__dict__.update(config_dict)

    # save config to file
    def _save(self):
        data = dict(self.__dict__)  # copy config
        data.pop("thread")  # remove the thread

        with open(cv("config.yml"), "w+") as f:
            dump(data, f)

    # automatically save any changes to the config
    def _update(self):
        thread = current_thread()

        while getattr(thread, "run", True):
            old_data = dict(self.__dict__)  # make copy of current data
            sleep(1)
            if old_data != self.__dict__:  # check for changes
                self._save()

    def stop(self):
        self.thread.run = False
