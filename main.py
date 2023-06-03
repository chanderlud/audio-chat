import imghdr
import sys
import tkinter.filedialog
from audioop import mul, rms
from base64 import b64encode
from hashlib import sha256
from io import BytesIO
from json import dump, load
from logging import debug, warning, error, getLogger
from multiprocessing import Process
from os import path, mkdir, listdir, fstat, makedirs
from random import randint
from socket import socket, AF_INET, timeout, SOCK_DGRAM, SOCK_STREAM
from subprocess import Popen, PIPE, DEVNULL
from sys import platform, exit
from tempfile import NamedTemporaryFile
from threading import Thread, current_thread
from time import sleep, time
from tkinter import font
from types import SimpleNamespace
from typing import List, Optional, Union, Tuple  # , Literal

import customtkinter as ctk
from Crypto.Cipher import AES
from PIL import Image, ImageGrab
from _tkinter import TclError
from func_timeout import func_timeout, FunctionTimedOut
from port_range import PortRange
from pyaudio import PyAudio, paInt16, Stream
from pydub import AudioSegment
from pydub.playback import play
from pystray import Icon, MenuItem

import widgets
from _nuitka import cv, download_path
from config import Config

# noise reduction only works on Windows for now
if platform == "win32":
    from rnnoise import RNNoiseVST

CHUNK = 480  # frames per buffer
FILE_CHUNK = 1024 * 1024  # bytes per file chunk
FORMAT = paInt16  # 16-bit signed audio data
CHANNELS = 1  # normal microphones are mono
RATE = 48000  # DVD quality
SILENCE = bytes([0] * (CHUNK * 2))  # a chunk of silent audio

GOODBYE_NOISE = AudioSegment.from_file("sounds/goodbye.wav", format="wav")
INCOMING_TONE = AudioSegment.from_file("sounds/incoming.wav", format="wav")
OUTGOING_TONE = AudioSegment.from_file("sounds/outgoing.wav", format="wav")
MESSAGE_NOISE = AudioSegment.from_file("sounds/message.wav", format="wav")
MUTE_NOISE = AudioSegment.from_file("sounds/mute.wav", format="wav")
UNMUTE_NOISE = AudioSegment.from_file("sounds/unmute.wav", format="wav")
DEAFEN_NOISE = AudioSegment.from_file("sounds/deafen.wav", format="wav")
UNDEAFEN_NOISE = AudioSegment.from_file("sounds/undeafen.wav", format="wav")
JOINED_NOISE = AudioSegment.from_file("sounds/joined.wav", format="wav")
LEFT_NOISE = AudioSegment.from_file("sounds/left.wav", format="wav")


class ByteBuffer(object):
    def __init__(self):
        self.buf = BytesIO()
        self.available = 0  # Bytes available for reading
        self.size = 0
        self.write_fp = 0

    def read(self, size=None):
        if size is None or size > self.available:
            size = self.available
        size = max(size, 0)

        result = self.buf.read(size)
        self.available -= size

        if len(result) < size:
            self.buf.seek(0)
            result += self.buf.read(size - len(result))

        return result

    def write(self, data):
        if self.size < self.available + len(data):
            # Expand buffer
            new_buf = BytesIO()
            new_buf.write(self.read())
            self.write_fp = self.available = new_buf.tell()
            read_fp = 0
            while self.size <= self.available + len(data):
                self.size = max(self.size, 1024) * 2
            new_buf.write(b"0" * (self.size - self.write_fp))
            self.buf = new_buf
        else:
            read_fp = self.buf.tell()

        self.buf.seek(self.write_fp)
        written = self.size - self.write_fp
        self.buf.write(data[:written])
        self.write_fp += len(data)
        self.available += len(data)
        if written < len(data):
            self.write_fp -= self.size
            self.buf.seek(0)
            self.buf.write(data[written:])
        self.buf.seek(read_fp)

    def seek(self, offset, whence=0):
        if whence == 0:
            # Seek from beginning
            self.buf.seek(offset)
            self.write_fp = self.available = self.buf.tell()
        elif whence == 1:
            # Seek from current position
            self.buf.seek(offset, whence)
            self.write_fp = self.available = self.buf.tell()
        elif whence == 2:
            # Seek from end
            self.buf.seek(offset, whence)
            self.write_fp = self.available = self.size - self.buf.tell()

    def tell(self):
        return self.buf.tell()


class FileTransfer(object):
    file_extension: str
    file_name: str
    file_length: int
    chunk_size: int
    signature: bytes

    def __init__(self, file_extension: str, file_name: str, file_length: int, chunk_size: int, signature: bytes):
        self.file_extension = file_extension
        self.file_name = file_name
        self.file_length = file_length
        self.chunk_size = chunk_size
        self.signature = signature

        debug(f"FileTransfer: {self.file_name}.{self.file_extension} "
              f"({self.file_length} bytes) (chunk = {self.chunk_size})"
              f" (signature = {self.signature.hex()})")

    @classmethod
    def from_handshake(cls, handshake: bytes):
        split = []
        offset = 0

        for _ in range(5):  # we expect five components
            length = int.from_bytes(handshake[offset:offset + 4], "big")
            offset += 4
            split.append(handshake[offset:offset + length])
            offset += length

        return cls(
            split[0].decode(),
            split[1].decode(),
            int.from_bytes(split[2], "big"),
            int.from_bytes(split[3], "big"),
            split[4]
        )

    @classmethod
    def from_file(cls, file_path: str):
        name, extension = parse_file_path(file_path)

        file = open(file_path, "rb")

        return cls(
            extension,
            name,
            fstat(file.fileno()).st_size,
            FILE_CHUNK,
            sha256(file.read()).digest()
        )

    def pack(self) -> bytes:
        components = [
            self.file_extension.encode(),
            self.file_name.encode(),
            self.file_length.to_bytes((self.file_length.bit_length() + 7) // 8, "big"),
            self.chunk_size.to_bytes((self.chunk_size.bit_length() + 7) // 8, "big"),
            self.signature
        ]

        packed = b""

        for comp in components:
            length = len(comp)
            packed += length.to_bytes(4, "big")  # prepend each component with its length
            packed += comp

        return packed

    @property
    def formatted_name(self):
        return f"{self.file_name}.{self.file_extension}"


class Contact(object):
    ip: str
    port: int
    secret: str
    nickname: str
    socket: socket
    online: bool
    latency: int
    thread: Thread

    def __init__(self, ip: str, port: int, secret: str, nickname: str):
        self.ip = ip
        self.port = port
        self.secret = secret
        self.nickname = nickname
        self.online = False
        self.latency = 0

        if len(self.secret) != 16:
            raise ValueError

        self.thread = Thread(target=self._ping)
        self.thread.start()

    def save(self) -> None:
        if not path.exists(cv("contacts")):
            mkdir(cv("contacts"))

        data = {
            "ip": self.ip,
            "port": self.port,
            "secret": self.secret,
            "nickname": self.nickname
        }

        with open(cv("contacts/" + self.nickname + ".json"), "w+") as f:
            dump(data, f)

    @staticmethod
    def load(file: str):
        with open(cv("contacts/" + file), "r") as f:
            data_dict = load(f)

        return Contact(
            data_dict["ip"],
            data_dict["port"],
            data_dict["secret"],
            data_dict["nickname"]
        )

    # perform hello handshake
    def say_hello(self, audio_range: str) -> (int, int):
        sock = self._connect()
        sock.settimeout(10)

        debug("sending hello start")
        sock.send(bytes([0]))
        response = sock.recv(37)
        _, buffer = unpack_message(self.secret.encode(), response)
        send_port = int.from_bytes(buffer, "big")
        debug(f"received send port {send_port}")

        receive_port = random_port(audio_range)
        message = pack_message(self.secret.encode(), 0, receive_port.to_bytes(2, "big"))
        sock.send(message)
        debug(f"send receive port {receive_port}")

        return send_port, receive_port

    # send a goodbye packet
    def say_goodbye(self, i: int) -> None:
        sock = self._connect()
        sock.send(bytes([i]))

    def screenshare_handshake(self, salt: bytes) -> int:
        sock = self._connect()
        sock.settimeout(10)

        sock.send(bytes([2]))
        buffer = sock.recv(37)
        _, response = unpack_message(self.secret.encode(), buffer)
        port = int.from_bytes(response, "big")

        srtp_key = b64_encode(self.secret.encode() + salt)

        message = pack_message(
            self.secret.encode(),
            2,
            f"""v=0
o=- 0 0 IN IP4 0.0.0.0
s=Audio Chat Screen Share
c=IN IP4 {self.ip}
t=0 0
a=tool:libavformat 59.31.100
m=video {port} RTP/AVP 96
b=AS:8000
a=rtpmap:96 H264/90000
a=fmtp:96 packetization-mode=1
a=crypto:1 AES_CM_128_HMAC_SHA1_80 inline:{srtp_key}""".encode()
        )
        sock.send(message)

        return port

    def file_transfer_handshake(self, transfer: FileTransfer) -> int:
        sock = self._connect()
        sock.send(bytes([4]))

        buffer = sock.recv(37)
        _, response = unpack_message(self.secret.encode(), buffer)
        port = int.from_bytes(response, "big")

        message = pack_message(
            self.secret.encode(),
            4,
            transfer.pack()
        )
        sock.send(message)

        return port

    def _ping(self) -> None:
        thread = current_thread()

        while getattr(thread, "run", True):
            now = time()

            try:
                sock = self._connect()
                self.online = True
                sock.close()
                elapsed = int((time() - now) * 1000)
                self.latency = elapsed
            except (OSError, TimeoutError, timeout):
                self.online = False

            sleep(1)

    def _connect(self) -> socket:
        sock = socket(AF_INET, SOCK_STREAM)
        sock.settimeout(5)
        sock.connect((self.ip, self.port))
        return sock

    def stop(self) -> None:
        self.thread.run = False


class AcceptState(object):
    def __init__(self):
        self._i = None

    def true(self):
        self._i = True

    def false(self):
        self._i = False

    @property
    def is_none(self) -> bool:
        return self._i is None

    @property
    def value(self):
        return self._i


class App(ctk.CTk):
    chat_config: Config
    p: PyAudio
    in_call: bool
    call_disconnected: bool
    in_screenshare: bool
    muted: bool
    deafened: bool
    output_frames: ByteBuffer
    input_frames: ByteBuffer
    key: bytes
    contacts: List[Contact]
    audio_socket: Optional[socket]
    audio_stream: Optional[Stream]
    tray_application: Optional[Icon]
    server: Thread

    def __init__(self):  # render most of gui
        super().__init__()

        self.chat_config = Config()
        self.p = PyAudio()
        self.in_call = False
        self.call_disconnected = False
        self.in_screenshare = False
        self.muted = False
        self.deafened = False
        self.output_frames = ByteBuffer()
        self.input_frames = ByteBuffer()
        self.key = bytes()
        self.contacts = load_contacts()
        self.audio_socket = None  # these values are only set while a call is active
        self.audio_stream = None
        self.tray_application = None  # tray application is only set while the app is minimized

        log = getLogger()
        log.setLevel(self.chat_config.debugLevel)

        self.geometry("840x725")
        self.minsize(840, 725)
        self.title("Audio Chat")
        self.protocol("WM_DELETE_WINDOW", self.hide_window)  # block window closing

        if platform == "win32":
            self.iconbitmap("assets/icon.ico")

        if platform == "darwin":  # macos
            self.defaultFont = font.Font(family="SF Display", name="SF Display", size=13)
        else:  # linux and windows
            self.defaultFont = font.Font(family="Roboto", name="Roboto", size=13)

        tkinter.Frame(self, height=32, bg="#222425").pack()  # spacer

        top_frame = ctk.CTkFrame(self)
        top_frame.pack(fill=ctk.X)

        tkinter.Frame(top_frame, width=32, bg="#222425").pack(side=tkinter.LEFT)  # spacer

        add_contact_frame = ctk.CTkFrame(top_frame, fg_color="#191919")
        add_contact_frame.pack(fill=ctk.BOTH, side=ctk.LEFT)

        tkinter.Frame(add_contact_frame, height=15, bg="#191919").pack()  # spacer

        tkinter.Label(
            add_contact_frame,
            text="Create Contact",
            bg="#191919",
            fg="#FFFFFF",
            font=(self.defaultFont.name, 20),
            anchor=ctk.W
        ).pack(padx=20, anchor=ctk.W)

        tkinter.Frame(add_contact_frame, height=5, bg="#191919").pack()  # spacer

        first_row = tkinter.Frame(add_contact_frame, bg="#191919")
        first_row.pack(expand=True, fill=tkinter.X)

        tkinter.Frame(first_row, width=20, bg="#191919").pack(side=tkinter.LEFT)  # spacer

        host_frame = tkinter.Frame(first_row, bg="#191919")
        host_frame.pack(side=tkinter.LEFT)

        tkinter.Label(
            host_frame,
            text="Host",
            bg="#191919",
            fg="#FFFFFF",
            font=(self.defaultFont.name, 15),
            anchor=ctk.W
        ).pack(pady=2, anchor=ctk.W)

        self.host_entry = ctk.CTkEntry(
            host_frame,
            border_width=0,
            fg_color="#27292A",
            placeholder_text="1.1.1.1",
            placeholder_text_color="#5d5f5f",
            text_color="#5d5f5f",
            font=(self.defaultFont.name, 13),
        )
        self.host_entry.pack(ipady=6, fill=ctk.X)

        tkinter.Frame(first_row, width=10, bg="#191919").pack(side=tkinter.LEFT)  # spacer

        port_frame = tkinter.Frame(first_row, bg="#191919")
        port_frame.pack(side=tkinter.LEFT)

        tkinter.Label(
            port_frame,
            text="Port",
            bg="#191919",
            fg="#FFFFFF",
            font=(self.defaultFont.name, 15),
            anchor=ctk.W
        ).pack(pady=2, anchor=ctk.W)

        self.port_entry = ctk.CTkEntry(
            port_frame,
            border_width=0,
            fg_color="#27292A",
            placeholder_text="8080",
            placeholder_text_color="#5d5f5f",
            text_color="#5d5f5f",
            font=(self.defaultFont.name, 13)
        )
        self.port_entry.pack(ipady=6, fill=ctk.X)

        tkinter.Frame(first_row, width=20, bg="#191919").pack(side=tkinter.LEFT)  # spacer

        second_row = tkinter.Frame(add_contact_frame, bg="#191919")
        second_row.pack(expand=True, fill=tkinter.X)

        tkinter.Frame(second_row, width=20, bg="#191919").pack(side=tkinter.LEFT)  # spacer

        secret_frame = tkinter.Frame(second_row, bg="#191919")
        secret_frame.pack(fill=tkinter.X, expand=True, side=tkinter.LEFT)

        tkinter.Label(
            secret_frame,
            text="Secret Key",
            bg="#191919",
            fg="#FFFFFF",
            font=(self.defaultFont.name, 15),
            anchor=ctk.W
        ).pack(pady=2, anchor=tkinter.W)

        self.secret_entry = ctk.CTkEntry(
            secret_frame,
            border_width=0,
            fg_color="#27292A",
            # placeholder_text_color="#5d5f5f",
            font=(self.defaultFont.name, 13)
        )
        self.secret_entry.pack(ipady=6, fill=ctk.X)

        tkinter.Frame(second_row, width=20, bg="#191919").pack(side=tkinter.LEFT)  # spacer

        bottom_row = tkinter.Frame(add_contact_frame, bg="#191919")
        bottom_row.pack(expand=True, fill=tkinter.X)

        tkinter.Label(
            bottom_row,
            text="Nickname",
            bg="#191919",
            fg="#FFFFFF",
            font=(self.defaultFont.name, 15),
            anchor=ctk.W
        ).pack(padx=20, pady=2, anchor=ctk.W)

        tkinter.Frame(bottom_row, width=20, bg="#191919").pack(side=tkinter.LEFT)  # spacer

        self.nickname_entry = ctk.CTkEntry(
            bottom_row,
            border_width=0,
            fg_color="#27292A",
            # placeholder_text_color="#5d5f5f",
            font=(self.defaultFont.name, 13)
        )
        self.nickname_entry.pack(ipady=6, fill=ctk.X, side=tkinter.LEFT, expand=True)

        tkinter.Frame(bottom_row, width=10, bg="#191919").pack(side=tkinter.LEFT)  # spacer

        ctk.CTkButton(bottom_row, text="Submit", command=self.add_contact).pack(ipady=5, side=tkinter.LEFT)

        tkinter.Frame(bottom_row, width=20, bg="#191919").pack(side=tkinter.LEFT)  # spacer

        tkinter.Frame(add_contact_frame, height=20, bg="#191919").pack()  # spacer

        tkinter.Frame(top_frame, width=32, bg="#222425").pack(side=tkinter.LEFT)  # spacer

        contacts_frame = ctk.CTkFrame(top_frame, fg_color="#191919")
        contacts_frame.pack(side=tkinter.LEFT, fill=ctk.BOTH, expand=True)

        tkinter.Frame(contacts_frame, height=15, bg="#191919").pack()  # spacer

        tkinter.Label(
            contacts_frame,
            text="Contacts",
            bg="#191919",
            fg="#FFFFFF",
            font=(self.defaultFont.name, 20)
        ).pack(padx=20, anchor=ctk.W)

        tkinter.Frame(contacts_frame, height=8, bg="#191919").pack()  # spacer

        self.contacts_placeholder = tkinter.Label(
            contacts_frame,
            text="Nothing here yet...",
            bg="#191919",
            fg="#5e5f5f",
            font=(self.defaultFont.name, 18)
        )
        self.contacts_placeholder.pack(anchor=tkinter.CENTER)

        canvas = tkinter.Canvas(contacts_frame, bg="#191919", highlightthickness=0)
        canvas.pack(side=ctk.LEFT, fill=ctk.BOTH, expand=True, pady=7)

        # create a frame widget inside the canvas widget to hold the contacts
        self.contacts_frame = tkinter.Frame(canvas, bg="#191919")

        # create a vertical scrollbar widget and connect it to the canvas widget
        scrollbar = ctk.CTkScrollbar(contacts_frame, command=canvas.yview, bg_color="transparent")
        scrollbar_spacer = tkinter.Frame(contacts_frame, bg="#191919")
        scrollbar_spacer.pack(side=ctk.RIGHT, fill=ctk.Y, pady=7, padx=11)
        canvas.configure(yscrollcommand=scrollbar.set)

        # add the frame widget to the canvas widget
        window = canvas.create_window((0, 0), window=self.contacts_frame, anchor=tkinter.NW)

        def frame_on_configure(_):
            canvas.update_idletasks()
            canvas.configure(scrollregion=canvas.bbox("all"))
            canvas.itemconfigure(window, width=canvas.winfo_width())

            # check if contacts_frame is smaller than canvas
            if canvas.winfo_height() >= self.contacts_frame.winfo_height():
                if scrollbar.winfo_viewable():
                    scrollbar.pack_forget()
                    scrollbar_spacer.pack(side=ctk.RIGHT, fill=ctk.Y, pady=7, padx=11)
            else:
                if scrollbar_spacer.winfo_viewable():
                    scrollbar_spacer.pack_forget()
                    scrollbar.pack(side=ctk.RIGHT, fill=ctk.Y, pady=7, padx=3)

        # allows scrolling & sets contacts_frame to width of canvas
        self.contacts_frame.bind("<Configure>", frame_on_configure)
        canvas.bind("<Configure>", frame_on_configure)

        # only bind MouseWheel events when cursor is over canvas
        canvas.bind("<Enter>", lambda _: bind_scroll(self.contacts_frame, canvas))
        canvas.bind("<Leave>", lambda _: unbind_scroll(self.contacts_frame))

        tkinter.Frame(top_frame, width=32, bg="#222425").pack(side=tkinter.LEFT)  # spacer

        for contact in self.contacts:  # render contacts
            self.render_contact(contact)

        self.after(1000, self.update_contact_status)  # start update loop

        tkinter.Frame(self, height=32, bg="#222425").pack()  # spacer

        bottom_row = tkinter.Frame(self, bg="#222425")
        bottom_row.pack(fill=tkinter.BOTH, expand=True)

        tkinter.Frame(bottom_row, width=32, bg="#222425").pack(side=tkinter.LEFT)  # spacer

        control_frame = ctk.CTkFrame(bottom_row, fg_color="#27292A", bg_color="#222425")
        control_frame.pack(fill=tkinter.Y, side=tkinter.LEFT)

        tkinter.Frame(control_frame, height=7, bg="#27292A").pack()  # spacer

        status_frame = tkinter.Frame(control_frame, bg="#27292A")
        status_frame.pack()

        tkinter.Frame(control_frame, height=2, bg="#27292A").pack()  # spacer

        ctk.CTkLabel(
            status_frame,
            text="Call Status: ",
            text_color="#c9c9ca",
            font=(self.defaultFont.name, 16)
        ).pack(side=ctk.LEFT)

        self.call_status = ctk.CTkLabel(
            status_frame,
            text="Ready",
            font=(self.defaultFont.name, 16)
        )
        self.call_status.pack(side=ctk.LEFT)

        ctk.CTkLabel(
            control_frame,
            text="Output Volume",
            font=(self.defaultFont.name, 14)
        ).pack(anchor=ctk.W, padx=25)

        output_frame = tkinter.Frame(control_frame, bg="#27292A")
        output_frame.pack(anchor=ctk.W, fill=tkinter.X, padx=1)

        tkinter.Frame(output_frame, width=20, bg="#27292A").pack(side=tkinter.LEFT)  # spacer

        slider = ctk.CTkSlider(
            output_frame,
            to=20,
            from_=-20,
            width=130,
            command=self.update_output_volume
        )
        slider.set(self.chat_config.outputVolume)
        slider.pack(side=ctk.LEFT)

        self.output_display = ctk.CTkLabel(
            output_frame,
            text=f"{self.chat_config.outputVolume} db"
        )
        self.output_display.pack(side=ctk.LEFT, padx=3)

        tkinter.Frame(control_frame, height=5, bg="#27292A").pack()  # spacer

        ctk.CTkLabel(
            control_frame,
            text="Input Volume",
            font=(self.defaultFont.name, 14)
        ).pack(anchor=ctk.W, padx=25)

        input_frame = tkinter.Frame(control_frame, bg="#27292A")
        input_frame.pack(anchor=ctk.W, fill=tkinter.X, padx=1)

        tkinter.Frame(input_frame, width=20, bg="#27292A").pack(side=tkinter.LEFT)  # spacer

        slider = ctk.CTkSlider(
            input_frame,
            to=20,
            from_=-20,
            width=130,
            command=self.update_input_volume
        )
        slider.set(self.chat_config.inputVolume)
        slider.pack(side=ctk.LEFT)

        self.input_display = ctk.CTkLabel(
            input_frame,
            text=f"{self.chat_config.inputVolume} db"
        )
        self.input_display.pack(side=ctk.LEFT, padx=3)

        tkinter.Frame(control_frame, height=5, bg="#27292A").pack()  # spacer

        ctk.CTkLabel(
            control_frame,
            text="Input Sensitivity",
            font=(self.defaultFont.name, 14)
        ).pack(anchor=ctk.W, padx=25)

        sensitivity_frame = tkinter.Frame(control_frame, bg="#27292A")
        sensitivity_frame.pack(anchor=ctk.W, fill=tkinter.X, padx=1)

        tkinter.Frame(sensitivity_frame, width=20, bg="#27292A").pack(side=tkinter.LEFT)  # spacer

        slider = ctk.CTkSlider(
            sensitivity_frame,
            to=50,
            from_=-16,
            width=130,
            command=self.update_input_sensitivity
        )
        slider.set(self.chat_config.inputSensitivity)
        slider.pack(side=ctk.LEFT)

        self.sensitivity_display = ctk.CTkLabel(
            sensitivity_frame,
            text=f"{self.chat_config.inputSensitivity} db"
        )
        self.sensitivity_display.pack(side=ctk.LEFT, padx=3)

        button_frame = ctk.CTkFrame(
            control_frame,
            fg_color="#191919",
            bg_color="#222425",
            background_corner_colors=("#191919", "#191919", "#222425", "#222425")
        )
        button_frame.pack(fill=tkinter.X, ipady=15, side=tkinter.BOTTOM)

        tkinter.Frame(button_frame, width=40, bg="#191919").pack(side=tkinter.LEFT)  # spacer

        self.mute_button = widgets.IconButton(button_frame, "assets/microphone.png", self.mute, "#191919")
        self.mute_button.pack(side=tkinter.LEFT)

        tkinter.Frame(button_frame, width=14, bg="#191919").pack(side=tkinter.LEFT)  # spacer

        self.deafen_button = widgets.IconButton(button_frame, "assets/deafen.png", self.deafen, "#191919")
        self.deafen_button.pack(side=tkinter.LEFT)

        tkinter.Frame(button_frame, width=17, bg="#191919").pack(side=tkinter.LEFT)  # spacer

        widgets.IconButton(button_frame, "assets/settings.png", self.settings, "#191919").pack(side=tkinter.LEFT)

        tkinter.Frame(button_frame, width=17, bg="#191919").pack(side=tkinter.LEFT)  # spacer

        self.screenshare_button = widgets.IconButton(
            button_frame,
            "assets/screenshare.png",
            self.screenshare,
            "#191919",
            disabled_icon="assets/screenshare_disabled.png"
        )
        self.screenshare_button.pack(side=tkinter.LEFT)
        self.screenshare_button.disable()

        tkinter.Frame(button_frame, width=40, bg="#191919").pack(side=tkinter.LEFT)  # spacer

        tkinter.Frame(bottom_row, width=32, bg="#222425").pack(side=tkinter.LEFT)  # spacer

        self.chat_widget = widgets.ChatWidget(bottom_row, fg_color="#191919")
        self.chat_widget.pack(side=tkinter.LEFT, fill=ctk.BOTH, expand=True)
        self.chat_widget.disable()  # chat is disabled until call is started

        tkinter.Frame(bottom_row, width=32, bg="#222425").pack(side=tkinter.RIGHT)  # spacer

        tkinter.Frame(self, height=32, bg="#222425").pack()  # spacer

        self.server = Thread(target=self.control_server)
        self.server.start()

    # functions for backend #

    def record_audio(self, output: ByteBuffer) -> None:
        silence_length = 0

        if self.chat_config.useRnnoise and platform == "win32":
            rnnoise = RNNoiseVST(CHUNK, RATE, self.chat_config)
        else:
            rnnoise = None  # make warnings go away

        while self.in_call:
            try:
                raw_data = self.audio_stream.read(CHUNK, exception_on_overflow=False)

                if self.muted:  # send silence packet
                    self.send_silence()
                    continue

                # denoise audio frame with rnnoise
                if rnnoise and self.chat_config.useRnnoise:
                    raw_data = rnnoise.process_frame(raw_data)
                elif not rnnoise and self.chat_config.useRnnoise and platform == "win32":
                    rnnoise = RNNoiseVST(CHUNK, RATE, self.chat_config)
                    raw_data = rnnoise.process_frame(raw_data)
                else:
                    rnnoise = None

                if rms(raw_data, 2) <= db_to_float(self.chat_config.inputSensitivity):  # frame is below threshold
                    # if silence is longer than 40 frames, start sending silence packets
                    if silence_length > 40:
                        self.send_silence()
                        continue
                    else:  # if silence is short process normally
                        silence_length += 1
                else:
                    silence_length = 0  # if not silent, silence length 0

                loud_data = mul(raw_data, 2, db_to_float(self.chat_config.inputVolume))  # adjust gain

                output.write(loud_data)
            except OSError as ex:
                error(f"Record error: {ex}")
                self.in_call = False
                break

    def play_audio(self, source: ByteBuffer) -> None:
        while self.in_call:
            if source.available > RATE * 0.25:
                debug(f"input stream is {source.available} bytes, clearing")
                source.read(source.available - (CHUNK * 2))

            if source.available >= (CHUNK * 2):
                frame_data = source.read(CHUNK * 2)

                if not self.deafened:
                    loud_data = mul(frame_data, 2, db_to_float(self.chat_config.outputVolume))

                    try:
                        self.audio_stream.write(loud_data)
                    except OSError:
                        continue
            else:
                sleep(0.05)

    def record_screenshare(self, command: str) -> None:
        p = Popen(  # open ffmpeg process
            command,
            stdin=PIPE,  # pipe stdin to ffmpeg for quiting
            shell=True,
            # hide ffmpeg output unless debug level is low enough
            stdout=DEVNULL if self.chat_config.debugLevel > 10 else None,
            stderr=DEVNULL if self.chat_config.debugLevel > 10 else None
        )

        while self.in_screenshare:  # block while screenshare is active
            sleep(0.05)

        debug("closing ffmpeg screenshare capturer")
        p.stdin.write(b"q")  # sending q to ffmpeg gracefully quits, usually works
        sleep(5)

        while p.poll() is None:  # need to send terminate multiple times
            p.terminate()

    def play_screenshare(self, sdp_data: bytes) -> None:
        sdp_file = NamedTemporaryFile("wb+", delete=False)  # create a temporary file to store sdp data
        sdp_file.write(sdp_data)  # write sdp data to file
        sdp_file.close()  # close file so it can be opened by ffplay

        p = Popen(  # open ffplay window for sdp file
            f"ffplay -fflags nobuffer -probesize 32 -analyzeduration 100000 -sync ext -protocol_whitelist rtp,udp,file {sdp_file.name}",
            stdin=PIPE,  # stdin can be used to close ffplay window (maybe)
            shell=True,
            # hide ffplay output unless debug level is low enough
            stdout=DEVNULL if self.chat_config.debugLevel > 10 else None,
            stderr=DEVNULL if self.chat_config.debugLevel > 10 else None
        )

        while self.in_screenshare:
            if p.poll() is None:  # check if playback window is still open
                sleep(0.05)
            else:  # end screenshare if playback window is closed
                self.end_screenshare()
                break

        debug(f"terminating screenshare playback window {p.returncode}")

        while p.poll() is None:  # need to send terminate multiple times
            p.terminate()

    def broadcast(self) -> None:
        while self.in_call:
            if self.output_frames.available > RATE * 0.25:
                self.output_frames.read(self.output_frames.available - (CHUNK * 2))
                debug(f"output stream is {self.output_frames.available} bytes, clearing")

            if self.output_frames.available >= CHUNK * 2 and self.audio_socket is not None:
                frame_data = self.output_frames.read(CHUNK * 2)
            else:
                sleep(0.05)
                continue

            buffer = pack_message(self.key, 0, frame_data)

            try:
                self.audio_socket.sendall(buffer)
            except (OSError, AttributeError) as ex:
                if not self.call_disconnected and self.in_call:
                    self.call_disconnected = True
                    self.chat_widget.add_message(None, "Call disconnected", False)
                    play_sound(LEFT_NOISE)

                error(f"Broadcast error: {ex}")
                sleep(0.1)
                continue

        debug(f"broadcaster exited")

    def receive(self, contact: Contact) -> None:
        message_buffer = ByteBuffer()

        while self.in_call:
            try:
                buffer, _ = self.audio_socket.recvfrom(CHUNK * 2 + 33)
            except (OSError, AttributeError) as ex:
                if not self.call_disconnected and self.in_call:
                    self.call_disconnected = True
                    self.chat_widget.add_message(None, "Call disconnected", False)
                    play_sound(LEFT_NOISE)

                error(f"Receiver error: {ex}")
                continue

            if self.call_disconnected and self.in_call:  # temporary disconnection
                self.call_disconnected = False
                self.chat_widget.add_message(None, "Call reconnected", False)
                play_sound(JOINED_NOISE)

            try:
                control_byte, plain_data = unpack_message(self.key, buffer)
            except ValueError as ex:
                error(f"Decrypt error: {ex}")
                continue

            if control_byte == 0:
                self.input_frames.write(plain_data)
            elif control_byte == 1:
                message_buffer.write(plain_data)

                if len(plain_data) < CHUNK * 2:
                    message = message_buffer.read().decode()
                    self.chat_widget.add_message(contact.nickname, message, False)
                    play_sound(MESSAGE_NOISE)
            elif control_byte == 3:  # received silence packet, play silent audio
                self.input_frames.write(SILENCE)

        debug(f"receiver exited")

    def receive_file(self, contact: Contact, port: int, transfer: FileTransfer) -> None:
        # receives a file over tcp

        sock = socket(AF_INET, SOCK_STREAM)
        sock.bind(("0.0.0.0", port))  # open port for receiving file

        sock.listen(1)  # no need for backlog, single connection only

        try:
            connection, client_address = sock.accept()  # accept connection
        except (OSError, TimeoutError, timeout) as ex:
            debug(f"receive file error: {ex}")
            return

        # insure connection is from expected source
        if client_address[0] != contact.ip:
            debug(f"receive file error: {client_address[0]} != {contact.ip}")
            return

        # add a message to chat window indicating file transfer has started
        self.chat_widget.add_message(contact.nickname, f"Receiving file {transfer.formatted_name}", False)
        file_buffer = ByteBuffer()  # buffer to hold file data

        while self.in_call:  # break transfer if call ends
            # a full message may be broken into multiple packets, so this buffer is used
            buffer = []

            while len(buffer) < transfer.chunk_size + 33:
                packet = connection.recv(transfer.chunk_size + 33 - len(buffer))

                # final packet may be smaller than chunk size
                if not packet:
                    break

                buffer.extend(packet)  # add packet to buffer

            # decrypt message and add to file buffer
            _, plain_data = unpack_message(self.key, bytes(buffer))

            if file_buffer.available + len(plain_data) < transfer.file_length:
                # if file is not fully received, add data to buffer
                file_buffer.write(plain_data)
            else:
                # file is fully received
                file_buffer.write(plain_data)
                debug(f"received full file {file_buffer.available}")

                file_data = file_buffer.read()  # read data from buffer
                signature = sha256(file_data).digest()  # calculate signature

                # insure signature matches
                if signature != transfer.signature:
                    warning(f"file signature mismatch: {signature} != {transfer.signature}")
                    break

                local_path = f"{download_path()}/{transfer.formatted_name}"

                open(local_path, "wb+").write(file_data)  # write file data to disk

                if imghdr.what(local_path):  # check if file is an image
                    try:
                        image = Image.open(local_path)  # open image
                        self.chat_widget.add_message(contact.nickname, image, False)  # display image
                    except Exception as ex:
                        error(f"Error displaying image: {ex}")
                else:  # file is not an image, display file message
                    self.chat_widget.file_message(contact.nickname, f"{transfer.formatted_name}")

                play_sound(MESSAGE_NOISE)  # play message sound
                break  # break out of loop

    def control_server(self) -> None:
        thread = current_thread()

        sock = socket(AF_INET, SOCK_STREAM)
        sock.bind(("0.0.0.0", self.chat_config.controlPort))  # open port for receiving control packets
        sock.settimeout(2)  # 2 second timeout for receiving packets

        sock.listen(1024)  # allow backlog for managing multiple connections

        while getattr(thread, "run", True):  # break loop if thread is stopped
            try:
                connection, client_address = sock.accept()
                packet = connection.recv(5)  # first packet is limited to 5 bytes
            except (OSError, TimeoutError, timeout):
                continue

            try:
                message_type = int(packet[0])  # message type is first byte
            except IndexError:
                continue

            debug(f"received control packet | len: {len(packet)} | message type: {message_type}")
            contact = self.get_contact(client_address[0])

            try:
                if message_type == 0:
                    debug("received hello")

                    # check if root window is visible, if not, make it visible
                    if not is_visible(self):
                        debug("running as tray app, making window visible")
                        if self.tray_application:
                            self.tray_application.stop()  # stop tray application if needed

                        self.deiconify()  # make window visible

                    if contact is None:
                        debug(f"ignored call from unknown host {client_address}")
                        continue
                    elif self.in_call and client_address[0] != self.audio_socket.getpeername()[0]:
                        debug(f"ignored call from {client_address}")
                        continue
                    elif not self.in_call:
                        debug("prompting for new call")

                        p = play_sound(INCOMING_TONE)

                        try:
                            accepted = func_timeout(10, self.call_alert, (contact.nickname,))
                        except FunctionTimedOut:
                            accepted = False

                        p.terminate()

                        if not accepted:
                            continue

                        self.key = contact.secret.encode()
                    else:
                        self.call_disconnected = False
                        debug(f"received call resume from {contact.nickname}")
                        self.chat_widget.add_message(None, "Call reconnected", False)
                        play_sound(JOINED_NOISE)

                    debug("starting port handshake")
                    # generate a random port to receive audio on
                    receive_port = random_port(self.chat_config.audioPorts)
                    message = pack_message(self.key, 0, receive_port.to_bytes(2, "big"))  # pack port into message
                    debug(f"sending receive port {receive_port}")
                    connection.send(message)  # send message to other client

                    packet = connection.recv(37)  # 1 control byte, 16 byte IV, 16 byte signature, 4 byte port
                    _, buffer = unpack_message(self.key, packet)  # unpack message, ignore control byte
                    send_port = int.from_bytes(buffer, "big")  # convert port to int
                    debug(f"received send port {send_port}")

                    if not self.in_call:
                        # the call was not connected before so full initialization is needed
                        try:
                            debug("configuring stream")
                            self.configure_stream()  # configure audio stream
                        except IndexError:  # if no audio devices are found, end call
                            sleep(0.1)
                            self.end_call(True, False, True, client_address[0])
                            self.base_alert("Critical error!", "No audio devices found, ending call.", "error")
                            continue

                        self.in_call = True
                        debug("starting call thread")
                        # start call thread
                        Thread(
                            target=self.audio_call,
                            args=(receive_port, client_address[0], send_port, contact)
                        ).start()
                    else:
                        # if the call was disconnected, reconnect it
                        debug("reconfiguring audio socket")

                        self.audio_socket = socket(AF_INET, SOCK_DGRAM)
                        self.audio_socket.settimeout(3)
                        self.audio_socket.bind(("0.0.0.0", receive_port))
                        self.audio_socket.connect((client_address[0], send_port))

                        debug("audio socket reconfigured")

                # these message types require a connected call and connection from the same address as the call
                elif self.audio_socket is not None and self.audio_socket.getpeername()[0] == client_address[0]:
                    if message_type == 1:  # call goodbye
                        debug("received call goodbye")
                        self.end_call(goodbye=False)
                        self.base_alert("Call ended", "Received goodbye from target", "info")
                    elif message_type == 2:  # screenshare handshake
                        debug("received screenshare handshake")

                        # generate a random port to receive video on
                        receive_port = random_port(self.chat_config.audioPorts)
                        message = pack_message(self.key, 2, receive_port.to_bytes(2, "big"))  # pack port into message
                        connection.send(message)  # send message to other client

                        buffer = connection.recv(2048)  # receive sdp
                        _, sdp_data = unpack_message(self.key, buffer)  # unpack sdp data, ignore control byte
                        Thread(target=self.play_screenshare, args=(sdp_data,)).start()  # start player
                        self.in_screenshare = True
                    elif message_type == 3:  # screenshare goodbye
                        debug("received screenshare goodbye")
                        self.in_screenshare = False
                        # change screenshare button back to start screenshare
                        self.screenshare_button.change_icon(SCREENSHARE)
                        self.screenshare_button.change_command(self.screenshare)
                    elif message_type == 4:  # file transfer handshake
                        debug("received file transfer handshake")

                        # generate a random port to receive *tcp* file data on
                        receive_port = random_port(self.chat_config.audioPorts)
                        message = pack_message(self.key, 2, receive_port.to_bytes(2, "big"))  # pack port into message
                        connection.send(message)  # send message to other client

                        buffer = connection.recv(2048)  # receive file transfer info
                        # unpack file transfer info, ignore control byte
                        _, handshake = unpack_message(self.key, buffer)

                        try:
                            transfer = FileTransfer.from_handshake(handshake)  # deserialize file transfer object
                            # start thread to receive file data
                            Thread(target=self.receive_file, args=(contact, receive_port, transfer)).start()
                        except IndexError:
                            debug("received invalid file transfer handshake")
                            continue
            except AttributeError:
                continue
            except ConnectionResetError:
                warning("connection reset error, aborting handshake")
                continue

    def audio_call(self, receive_port: int, send_host: str, send_port: int, contact: Contact) -> None:
        self.enable_call_buttons(False)
        self.set_end_call_button(contact, True)
        self.screenshare_button.enable()
        self.chat_widget.enable()

        self.chat_widget.clear()
        self.chat_widget.add_message(None, "Chat connected!", False)
        self.call_status.configure(text="Connected")

        self.audio_socket = socket(AF_INET, SOCK_DGRAM)
        self.audio_socket.settimeout(3)
        self.audio_socket.bind(("0.0.0.0", receive_port))
        self.audio_socket.connect((send_host, send_port))

        Thread(target=self.record_audio, daemon=True, args=(self.output_frames,)).start()
        Thread(target=self.play_audio, daemon=True, args=(self.input_frames,)).start()

        debug(f"listening for audio packets udp://0.0.0.0:{receive_port}")
        debug(f"sending audio packets to udp://{send_host}:{send_port}")

        Thread(target=self.receive, args=(contact,)).start()
        Thread(target=self.broadcast).start()

        while self.in_call:
            sleep(0.05)

        if self.audio_stream:
            debug("closing audio stream")
            self.audio_stream.close()
            self.audio_stream = None
            debug("audio stream closed")

        self.enable_call_buttons(True)
        self.set_end_call_button(contact, False)
        self.screenshare_button.disable()
        self.chat_widget.disable()

        self.chat_widget.clear()
        self.chat_widget.add_message(None, "Chat Disconnected!", False)
        self.call_status.configure(text="Ended")
        debug("call ended")

    def initiate_call(self, contact: Contact) -> None:
        if self.in_call:
            return

        self.in_call = True

        if self.audio_stream:
            error("audio stream is some at start of call")
            self.audio_stream.close()

        try:
            self.configure_stream()
        except IndexError:
            self.in_call = False
            self.base_alert("Critical error!", "No audio devices found.", "error")
            self.call_status.configure(text="Ready")
            return

        self.call_status.configure(text="Connecting")
        p = play_sound(OUTGOING_TONE)

        try:
            send_port, receive_port = contact.say_hello(self.chat_config.audioPorts)
            self.key = contact.secret.encode()
            p.terminate()
            Thread(target=self.audio_call, args=(receive_port, contact.ip, send_port, contact)).start()
        except (timeout, TimeoutError, ConnectionRefusedError, OSError):
            p.terminate()
            self.in_call = False
            self.base_alert("Contact Offline", "Target host does not seem to be online", "error")
            self.call_status.configure(text="Ready")
        except IndexError:
            p.terminate()
            self.in_call = False
            self.base_alert("An Error Occurred", "Please try to make your call again", "error")
            self.call_status.configure(text="Ready")

    def configure_stream(self) -> None:
        self.audio_stream = self.p.open(
            format=FORMAT,
            channels=CHANNELS,
            rate=RATE,
            input=True,
            output=True,
            frames_per_buffer=CHUNK,
            input_device_index=self.input_device_index,
            output_device_index=self.output_device_index
        )

    def audio_test(self) -> None:
        if self.in_call:  # can't do test if in call
            return

        self.in_call = True

        try:
            self.configure_stream()
        except IndexError:
            self.end_call(False, False, True)
            self.base_alert("Critical error!", "No audio devices found, ending audio test.", "error")
            return

        self.call_status.configure(text="Audio Test")

        Thread(target=self.record_audio, daemon=True, args=(self.output_frames,)).start()
        # reroute output frames into input frames
        Thread(target=self.play_audio, daemon=True, args=(self.output_frames,)).start()

    def send_silence(self) -> None:
        try:
            self.audio_socket.sendall(bytes([3]))
        except AttributeError:
            return

    def get_contact(self, ip: str) -> Optional[Contact]:
        for contact in self.contacts:
            if contact.ip == ip:
                return contact

        return None

    def deafen(self) -> None:
        if self.muted and not self.deafened:
            return
        elif self.deafened:
            self.deafened = False
            self.muted = False

            # restore original icons
            self.deafen_button.change_icon(DEAFEN)
            self.mute_button.change_icon(MICROPHONE)

            play_sound(UNDEAFEN_NOISE)
        else:
            self.deafened = True
            self.muted = True

            # change to deafened and muted icons
            self.deafen_button.change_icon(UNDEAFEN)
            self.mute_button.change_icon(MUTED_MICROPHONE)

            play_sound(DEAFEN_NOISE)

    def mute(self) -> None:
        if self.deafened:
            return
        elif self.muted:
            self.muted = False
            self.mute_button.change_icon(MICROPHONE)  # restore original icon
            play_sound(UNMUTE_NOISE)
        else:
            self.muted = True
            self.mute_button.change_icon(MUTED_MICROPHONE)  # change to unmute icon
            play_sound(MUTE_NOISE)

    def send_message(self) -> None:
        if self.audio_socket is None:  # cannot send message if not in call
            return

        message = self.chat_widget.get_entry()
        attachments = self.chat_widget.take_attachments()

        if message:
            self.chat_widget.add_message("You", message, True)

            # split message into chunks that fit in socket
            raw_data = message.encode()
            for chunk in [raw_data[i:i + CHUNK * 2] for i in range(0, len(raw_data), CHUNK * 2)]:
                buffer = pack_message(self.key, 1, chunk)
                self.audio_socket.sendall(buffer)

            for attachment in attachments:
                Thread(target=self.send_file, args=(attachment,)).start()

        elif not message and len(attachments) > 0:
            for attachment in attachments:
                Thread(target=self.send_file, args=(attachment,)).start()

    def add_attachment(self, mode: str) -> None:
        if self.audio_socket is None:  # cannot add attachment if not in call
            return

        if mode == "file":
            file_path = tkinter.filedialog.askopenfilename()

            if not file_path:
                return
        else:
            try:
                clipboard_contents = self.clipboard_get()

                if path.isfile(clipboard_contents):
                    file_path = clipboard_contents
                else:  # clipboard contains normal text
                    if platform == "darwin":  # on win32 this is not needed
                        self.chat_widget.message_entry.insert(ctk.END, clipboard_contents)
                    return
            except TclError:
                image = ImageGrab.grabclipboard()

                if not image:
                    return

                f = NamedTemporaryFile("wb+", delete=False)
                file_path = f.name
                image.save(file_path, format="png")

        self.chat_widget.add_attachment(file_path)

    def send_file(self, file_path: str) -> None:
        name, extension = parse_file_path(file_path)

        if imghdr.what(file_path):
            image = Image.open(file_path)
            self.chat_widget.add_message("You", image, False)
        else:
            self.chat_widget.file_message("You", f"{name}.{extension}", file_path)

        transfer = FileTransfer.from_file(file_path)
        send_port = self.connected_contact.file_transfer_handshake(transfer)

        sock = socket(AF_INET, SOCK_STREAM)
        sock.settimeout(5)
        sock.connect((self.connected_contact.ip, send_port))

        file = open(file_path, "rb")

        while True:
            data = file.read(transfer.chunk_size)

            if not data:
                debug("sent entire file")
                break

            buffer = pack_message(self.key, 2, data, check=False)
            sock.sendall(buffer)

    def update_input_sensitivity(self, value: float):
        rounded = round(value, 1)
        self.sensitivity_display.configure(text=f"{rounded} db")
        self.chat_config.inputSensitivity = rounded

    def update_output_volume(self, value: float):
        rounded = round(value, 1)
        self.output_display.configure(text=f"{rounded} db")
        self.chat_config.outputVolume = rounded

    def update_input_volume(self, value: float):
        rounded = round(value, 1)
        self.input_display.configure(text=f"{rounded} db")
        self.chat_config.inputVolume = rounded

    @property
    def input_device_index(self) -> int:
        devices = get_devices(self.p, "maxInputChannels")

        for device in devices:
            if device.name == self.chat_config.inputDevice:
                return device.index

        warning("selected input device not found")
        default_device = devices[0]
        self.chat_config.inputDevice = default_device.name
        return default_device.index

    @property
    def output_device_index(self) -> int:
        devices = get_devices(self.p, "maxOutputChannels")

        for device in devices:
            if device.name == self.chat_config.outputDevice:
                return device.index

        warning("selected output device not found")
        default_device = devices[0]
        self.chat_config.outputDevice = default_device.name
        return default_device.index

    @property
    def connected_contact(self) -> Optional[Contact]:
        if not self.audio_socket:
            return None

        other_ip = self.audio_socket.getpeername()[0]
        return self.get_contact(other_ip)

    # GUI functions only below this point #

    def settings(self) -> None:
        widgets.SettingsWidget(self, self.chat_config, self.p)  # run the settings window widget

    def screenshare(self) -> None:
        if self.audio_socket is not None:
            widgets.ScreenshareWidget(self)

    def base_alert(self, title: str, text: str,
                   alert_type  # : Literal["info", "error"]
                   ) -> None:
        alert = ctk.CTkToplevel(self, fg_color="#27292A")  # top level window
        alert.geometry("500x125")
        alert.resizable(False, False)
        alert.title(title)

        if platform == "win32":
            if alert_type == "info":
                alert.iconbitmap("assets/info.ico")
            else:
                alert.iconbitmap("assets/error.ico")

        alert.geometry(f"+{self.winfo_x() + 100}+{self.winfo_y() + 100}")  # controls placement of alert
        alert.wm_transient(self)  # makes alert appear on top of main window

        tkinter.Frame(alert, width=30, bg="#27292A").pack(side=tkinter.LEFT)  # spacer

        if alert_type == "info":
            image = tkinter.PhotoImage(file="assets/info.png")
        else:
            image = tkinter.PhotoImage(file="assets/error.png")

        # the main icon
        icon_label = tkinter.Label(alert, image=image, bg="#27292A")
        icon_label.pack(side=tkinter.LEFT)
        icon_label.image = image

        tkinter.Frame(alert, width=20, bg="#27292A").pack(side=tkinter.LEFT)  # spacer

        frame = tkinter.Frame(alert, bg="#27292A")  # frame the holds text and button
        frame.pack(side=tkinter.LEFT, anchor=tkinter.CENTER)

        ctk.CTkLabel(
            frame,
            text=text,
            font=(self.defaultFont.name, 20)
        ).pack(padx=10, pady=10, anchor=ctk.W)

        ctk.CTkButton(
            frame,
            text="Ok",
            command=alert.destroy  # destroy alert when acknowledged
        ).pack(padx=10, pady=10, anchor=ctk.W)

    def call_alert(self, caller: str) -> bool:
        alert = ctk.CTkToplevel(self, fg_color="#27292A")
        alert.title("Incoming Call")
        alert.geometry("500x115")
        alert.resizable(False, False)

        if platform == "win32":
            alert.iconbitmap("assets/icon.ico")

        alert.geometry(f"+{self.winfo_x() + 100}+{self.winfo_y() + 100}")  # position alert

        accepted = AcceptState()  # the state for the alert

        def accept():  # accept the call command
            accepted.true()
            alert.destroy()

        def reject():  # reject the call command
            accepted.false()
            alert.destroy()

        tkinter.Frame(alert, width=30, bg="#27292A").pack(side=tkinter.LEFT)  # spacer

        # the main icon
        image = tkinter.PhotoImage(file="assets/call_medium.png")
        icon_label = tkinter.Label(alert, image=image, bg="#27292A")
        icon_label.pack(side=tkinter.LEFT)
        icon_label.image = image

        tkinter.Frame(alert, width=20, bg="#27292A").pack(side=tkinter.LEFT)  # spacer

        ctk.CTkLabel(
            alert,
            text=f"Incoming call from {caller}",
            font=(self.defaultFont.name, 20)
        ).pack(padx=10, pady=10, side=ctk.TOP, anchor=tkinter.W)

        frame = tkinter.Frame(alert, bg="#27292A")  # frame the holds the buttons
        frame.pack(side=tkinter.LEFT, anchor=tkinter.CENTER)

        ctk.CTkButton(frame, text="Ignore", command=reject).pack(padx=5, pady=10, side=ctk.LEFT)
        ctk.CTkButton(frame, text="Accept", command=accept).pack(padx=5, pady=10, side=ctk.LEFT)

        while accepted.is_none:  # wait for alert to get accepted or rejected
            sleep(0.01)

        return accepted.value  # return the value of the alert

    def add_contact(self) -> None:
        host = self.host_entry.get()

        try:
            port = int(self.port_entry.get())
        except ValueError:
            self.base_alert("Invalid port", "Target port should be a valid integer", "error")
            return

        secret = self.secret_entry.get()
        nickname = self.nickname_entry.get()

        try:
            contact = Contact(host, port, secret, nickname)
            self.contacts.append(contact)
            self.render_contact(contact)
            contact.save()
        except ValueError:
            self.base_alert("Invalid secret key", "Secret key should be a 16 character string", "error")
            return

        # clear entries
        self.host_entry.delete(0, "end")
        self.secret_entry.delete(0, "end")
        self.port_entry.delete(0, "end")
        self.nickname_entry.delete(0, "end")

    def render_contact(self, contact: Contact) -> None:
        if self.contacts_placeholder:
            self.contacts_placeholder.destroy()
            self.contacts_placeholder = None

        master_frame = tkinter.Frame(self.contacts_frame, bg="#191919")
        master_frame.pack(fill=ctk.X, expand=True)

        tkinter.Frame(master_frame, bg="#191919", width=25).pack(side=ctk.LEFT)  # spacer

        contact_frame = ctk.CTkFrame(master_frame, fg_color="#27292A")
        contact_frame.pack(fill=ctk.X, expand=True, ipady=8, side=ctk.LEFT)

        tkinter.Frame(contact_frame, bg="#27292A", width=7).pack(side=ctk.LEFT)  # spacer

        image = tkinter.PhotoImage(file="assets/contact_offline.png")
        icon_label = tkinter.Label(contact_frame, image=image, bg="#27292A")
        icon_label.pack(side=tkinter.LEFT, padx=7)
        icon_label.image = image

        text_group = tkinter.Frame(contact_frame, bg="#27292A", width=150, height=50)
        text_group.pack(side=tkinter.LEFT)

        ctk.CTkLabel(
            text_group,
            text=contact.nickname,
            font=(self.defaultFont.name, 17)
        ).place(x=0, y=-3)

        ctk.CTkLabel(
            text_group,
            text=f"{contact.ip}:{contact.port}",
            font=(self.defaultFont.name, 12)
        ).place(x=0, y=20 if platform == "darwin" else 17)

        tkinter.Frame(contact_frame, bg="#27292A").pack(side=ctk.LEFT, fill=ctk.X, expand=True)  # spacer

        ctk.CTkLabel(contact_frame, text=f"{contact.latency}ms").pack(side=tkinter.LEFT)

        widgets.IconButton(
            contact_frame,
            "assets/call.png",
            lambda: Thread(target=self.initiate_call, args=(contact,)).start(),
            disabled_icon="assets/call_disabled.png"
        ).pack(side=tkinter.LEFT, padx=15)

        tkinter.Frame(self.contacts_frame, height=5, bg="#191919").pack()  # spacer

    def enable_call_buttons(self, enabled: bool) -> None:
        for master_frame in self.contacts_frame.winfo_children():
            try:
                contact_frame = master_frame.winfo_children()[1]
            except IndexError:
                continue

            try:
                button: widgets.IconButton = contact_frame.winfo_children()[5]
            except IndexError:
                continue

            if enabled:
                button.enable()
            else:
                button.disable()

    def set_end_call_button(self, contact: Contact, enable: bool) -> None:
        for master_frame in self.contacts_frame.winfo_children():
            try:
                contact_frame = master_frame.winfo_children()[1]
            except IndexError:
                continue

            widgets = contact_frame.winfo_children()

            try:
                if widgets[2].winfo_children()[0].cget("text") != contact.nickname:
                    continue
            except (IndexError, TclError):
                continue

            button: widgets.IconButton = widgets[5]

            if enable:
                button.change_icon(END_CALL)
                button.enable()
                button.change_command(self.end_call)
            else:
                button.change_icon(CALL)
                button.change_command(lambda: Thread(target=self.initiate_call, args=(contact,)).start())

    def update_contact_status(self) -> None:
        for contact in self.contacts:
            for master_frame in self.contacts_frame.winfo_children():
                try:
                    contact_frame = master_frame.winfo_children()[1]
                except IndexError:
                    continue

                widgets = contact_frame.winfo_children()

                try:
                    if widgets[2].winfo_children()[0].cget("text") != contact.nickname:
                        continue
                except (IndexError, TclError):
                    continue

                if contact.online:
                    widgets[1].configure(image=CONTACT_ONLINE)
                    widgets[1].image = CONTACT_ONLINE
                else:
                    widgets[1].configure(image=CONTACT_OFFLINE)
                    widgets[1].image = CONTACT_OFFLINE

                widgets[4].configure(text=f"{contact.latency}ms")

        self.after(1000, self.update_contact_status)

    def end_call(self, goodbye=True, sound=True, force=False, host=None) -> None:
        if self.audio_socket is None and not force:
            return

        debug("ending call")

        if sound:
            play_sound(GOODBYE_NOISE)

        self.output_frames = ByteBuffer()
        self.input_frames = ByteBuffer()

        if goodbye:
            if not host:
                contact = self.connected_contact
            else:
                contact = self.get_contact(host)

            try:
                contact.say_goodbye(1)
            except (ConnectionRefusedError, timeout, TimeoutError, OSError) as ex:
                debug(f"error sending goodbye {ex}")

        self.audio_socket = None
        self.in_call = False
        self.in_screenshare = False

        debug("setting call status")
        self.call_status.configure(text="Ready")
        debug("end call completed")

    def end_screenshare(self) -> None:
        if not self.in_screenshare:
            return

        try:
            self.connected_contact.say_goodbye(3)
        except (ConnectionRefusedError, timeout, TimeoutError, OSError):
            pass

        self.screenshare_button.change_icon(SCREENSHARE)
        self.screenshare_button.change_command(self.screenshare)

        self.in_screenshare = False

    def spin_down(self) -> None:
        debug("spinning down")

        self.in_call = False
        self.in_screenshare = False
        self.chat_config.stop()
        self.server.run = False

        if self.audio_stream:
            self.audio_stream.close()
            self.audio_stream = None

        self.p.terminate()

        for contact in self.contacts:
            contact.stop()

        if self.tray_application:
            self.tray_application.stop()
        self.destroy()
        exit()

    def show_window(self) -> None:
        if not self.winfo_viewable():
            self.tray_application.stop()
            self.deiconify()

    def hide_window(self) -> None:
        if self.chat_config.trayApp:
            if self.winfo_viewable():
                self.withdraw()

                self.tray_application = Icon(
                    "name", Image.open("assets/icon.ico"),
                    "Audio Chat", (
                        MenuItem("Show", self.show_window),
                        MenuItem("Quit", self.spin_down)
                    )
                )
                self.tray_application.run()
        else:
            self.spin_down()


def get_devices(p: PyAudio, device_filter: str) -> List[SimpleNamespace]:
    devices = []
    info = p.get_default_host_api_info()
    count = info.get("deviceCount")

    for i in range(0, count):
        device = p.get_device_info_by_host_api_device_index(0, i)
        if device.get(device_filter) > 0:
            devices.append(
                SimpleNamespace(**device)
            )

    if len(devices) == 0:
        raise IndexError

    return devices


# random port for audio
def random_port(port_range: str) -> int:
    actual_range = PortRange(port_range)

    return randint(actual_range.port_from, actual_range.port_to)


def load_contacts() -> List[Contact]:
    contacts = []

    if not path.exists(cv("contacts")):
        makedirs(cv("contacts"))

    for contact_file in listdir(cv("contacts")):
        contacts.append(
            Contact.load(contact_file)
        )

    return contacts


# play a pydub AudioSegment without blocking
def play_sound(sound: AudioSegment) -> Process:
    process = Process(target=play, args=(sound,))
    process.start()
    return process


# https://github.com/jiaaro/pydub/blob/master/pydub/utils.py#L79
def db_to_float(db: float, using_amplitude=True):
    """
    Converts the input db to a float, which represents the equivalent
    ratio in power.
    """
    if using_amplitude:
        return 10 ** (db / 20)
    else:  # using power
        return 10 ** (db / 10)


def unpack_message(key: bytes, buffer: bytes) -> Tuple[int, Optional[bytes]]:
    control_byte = buffer[0]

    if control_byte == 3:  # received silence packet, play silent audio
        return 3, None

    nonce = buffer[1:17]
    tag = buffer[17:33]
    ciphertext = buffer[33:]

    cipher = AES.new(key, AES.MODE_EAX, nonce=nonce)
    plain_data = cipher.decrypt(ciphertext)
    cipher.verify(tag)

    return control_byte, plain_data


def pack_message(key: bytes, control: int, message: bytes, check=True) -> bytes:
    if len(message) > CHUNK * 2 and check:
        raise ValueError

    cipher = AES.new(key, AES.MODE_EAX)
    ciphertext, tag = cipher.encrypt_and_digest(message)

    buffer = [control]
    buffer.extend(cipher.nonce)
    buffer.extend(tag)
    buffer.extend(ciphertext)

    return bytes(buffer)


def parse_file_path(file_path: str) -> Tuple[str, str]:
    _, full_file_name = path.split(file_path)
    name, extension = path.splitext(full_file_name)
    return name, extension.strip(".")


def is_visible(widget: any) -> bool:
    # winfo_viewable blocks if the window is not visible, so it times out
    try:
        _ = func_timeout(0.1, widget.winfo_viewable)  # always returns true
        return True
    except FunctionTimedOut:
        return False


# wrapper for base64 encoding that accepts bytes and strings
def b64_encode(i: Union[str, bytes]) -> str:
    if isinstance(i, str):
        i = i.encode()

    return b64encode(i).decode()


def scroll_canvas(event: tkinter.Event, canvas: tkinter.Canvas) -> None:
    canvas.yview_scroll(
        -1 * (event.delta // 120 if platform == "win32" else event.delta),
        "units"
    )


def bind_scroll(frame: tkinter.Frame, canvas: tkinter.Canvas) -> None:
    if canvas.winfo_height() < frame.winfo_height():  # only need scrolling if the frame is taller than the canvas
        frame.bind_all(
            "<MouseWheel>",
            lambda event: scroll_canvas(event, canvas)
        )


def unbind_scroll(frame: tkinter.Frame) -> None:
    frame.unbind_all("<MouseWheel>")


if __name__ == "__main__":
    # redirect stdout and stderr to files
    sys.stdout = open(cv("stdout.log"), "w+")
    sys.stderr = open(cv("stderr.log"), "w+")

    ctk.set_default_color_theme("assets/ctk_theme.json")
    ctk.set_appearance_mode("dark")

    audio_chat = App()

    # tkinter stuff is initialized so load images
    MICROPHONE = tkinter.PhotoImage(file="assets/microphone.png")
    MUTED_MICROPHONE = tkinter.PhotoImage(file="assets/muted_microphone.png")
    DEAFEN = tkinter.PhotoImage(file="assets/deafen.png")
    UNDEAFEN = tkinter.PhotoImage(file="assets/undeafen.png")
    SCREENSHARE = tkinter.PhotoImage(file="assets/screenshare.png")
    END_SCREENSHARE = tkinter.PhotoImage(file="assets/end_screenshare.png")
    SETTINGS = tkinter.PhotoImage(file="assets/settings.png")
    SEND = tkinter.PhotoImage(file="assets/send.png")
    FILE = tkinter.PhotoImage(file="assets/file.png")
    CALL = tkinter.PhotoImage(file="assets/call.png")
    END_CALL = tkinter.PhotoImage(file="assets/end_call.png")
    CONTACT_OFFLINE = tkinter.PhotoImage(file="assets/contact_offline.png")
    CONTACT_ONLINE = tkinter.PhotoImage(file="assets/contact_online.png")

    audio_chat.mainloop()
