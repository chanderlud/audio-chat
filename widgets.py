import secrets
import tkinter
import webbrowser
from logging import debug
from re import compile
from subprocess import Popen
from sys import platform
from threading import Thread
from typing import Union, Callable, List, Optional, Tuple

import customtkinter as ctk
from PIL import Image, ImageTk, ImageDraw
from port_range import PortRange
from pyaudio import PyAudio

# windows only imports
if platform == "win32":
    import ctypes
    import win32gui
    from ctypes import wintypes
    import win32con

import main

URL_REGEX = compile(r"\bhttps?://\S+\b")


# icon button widget with consistent look across platforms
class IconButton(tkinter.Button if platform == "win32" else tkinter.Label):
    icon: tkinter.PhotoImage
    command: Callable[[], None]
    _disabled: bool
    _disabled_icon: Optional[tkinter.PhotoImage]

    def __init__(self,
                 master: any,
                 icon: Union[tkinter.PhotoImage, str],
                 command: Callable[[], None],
                 bg: str = "#27292A",
                 disabled_icon: Union[tkinter.PhotoImage, str] = None,
                 **kwargs):

        if isinstance(icon, str):
            icon = tkinter.PhotoImage(file=icon)

        if disabled_icon:
            if isinstance(disabled_icon, str):
                disabled_icon = tkinter.PhotoImage(file=disabled_icon)

            self._disabled_icon = disabled_icon
        else:
            self._disabled_icon = None

        super().__init__(
            master,
            image=icon,
            cursor="pointinghand" if platform == "darwin" else "hand2",
            bg=bg,
            **kwargs
        )

        if platform == "win32":
            self.configure(borderwidth=0, activebackground=bg, command=command)

        self.bind("<Button-1>", self._on_click)
        self.icon = icon
        self.command = command
        self._disabled = False

    def change_icon(self, icon: Union[tkinter.PhotoImage, str]) -> None:
        if isinstance(icon, str):
            icon = tkinter.PhotoImage(file=icon)

        self.configure(image=icon)
        self.icon = icon

    def change_command(self, command: Callable[[], None]) -> None:
        self.command = command

        if platform == "win32":
            self.configure(command=command)

    def disable(self) -> None:
        self._disabled = True
        self.configure(cursor="arrow")

        if not self._disabled_icon:
            return  # no disabled icon

        if platform == "win32":
            self.configure(command=None, image=self._disabled_icon)
        else:
            self.configure(image=self._disabled_icon)

    def enable(self) -> None:
        self._disabled = False
        self.configure(cursor="pointinghand" if platform == "darwin" else "hand2")

        if platform == "win32":
            self.configure(command=self.command, image=self.icon)
        else:
            self.configure(image=self.icon)

    def _on_click(self, _event: tkinter.Event) -> Optional[str]:
        if platform != "win32" and not self._disabled:  # run command if not disabled
            self.command()
        elif platform == "win32" and self._disabled:  # blocks animation when disabled
            return "break"


class ChatWidget(ctk.CTkFrame):
    _images: List[ImageTk.PhotoImage]
    _attachments: List[str]
    _canvas: tkinter.Canvas
    _messages: tkinter.Frame
    message_entry: tkinter.Text
    _message_scrollbar: ctk.CTkScrollbar
    _last_nickname: Optional[str]

    def __init__(self, master: tkinter.Frame, **kwargs):
        super().__init__(master, **kwargs)

        self._attachments = []
        self.defaultFont = self.master.master.defaultFont
        self._last_nickname = None

        # create a canvas widget to hold the contents of the chat
        message_frame = ctk.CTkFrame(self, bg_color="#222425", fg_color="#27292A")
        message_frame.pack(fill=ctk.BOTH, expand=True)

        tkinter.Frame(message_frame, height=8, bg="#27292A").pack()  # spacer

        self._canvas = tkinter.Canvas(message_frame, bg="#27292A", highlightthickness=0)
        self._canvas.pack(side=ctk.LEFT, fill=ctk.BOTH, expand=True)

        # create a frame widget inside the canvas widget to hold the messages
        self._messages = tkinter.Frame(self._canvas, bg="#27292A")
        # top padding inside frame widget
        tkinter.Frame(self._messages, height=5, bg="#27292A").pack()

        # create a vertical scrollbar widget and connect it to the canvas widget
        scrollbar = ctk.CTkScrollbar(message_frame, command=self._canvas.yview, bg_color="transparent")
        self._canvas.configure(yscrollcommand=scrollbar.set)

        # add the frame widget to the canvas widget
        window = self._canvas.create_window((0, 0), window=self._messages, anchor=tkinter.NW)

        def frame_on_configure(_):
            self._canvas.update_idletasks()
            self._canvas.configure(scrollregion=self._canvas.bbox("all"))
            self._canvas.itemconfigure(window, width=self._canvas.winfo_width())

            # check if _messages is smaller than canvas
            if self._canvas.winfo_height() >= self._messages.winfo_height():
                scrollbar.pack_forget()
            else:
                scrollbar.pack(side=ctk.RIGHT, fill=ctk.Y, padx=4)

        # controls some stuff
        self._messages.bind("<Configure>", frame_on_configure)
        self._canvas.bind("<Configure>", frame_on_configure)

        # prevents a bug on windows
        self.bind("<Configure>", lambda _: self._canvas.configure(height=self.winfo_height() - 132))

        # only bind MouseWheel events when cursor is over canvas
        self._canvas.bind("<Enter>", lambda _: main.bind_scroll(self._messages, self._canvas))
        self._canvas.bind("<Leave>", lambda _: main.unbind_scroll(self._messages))

        self._entry_frame = ctk.CTkFrame(self, fg_color="#27292A")
        self._entry_frame.pack(anchor=ctk.S, pady=15, padx=25, fill=ctk.X)

        tkinter.Frame(self._entry_frame, width=10, bg="#27292A").pack(side=tkinter.LEFT)  # spacer

        self.message_entry = tkinter.Text(
            self._entry_frame,
            bg="#27292A",
            borderwidth=0,
            highlightthickness=0,
            height=1,
            width=1,  # this is a minimum width for some reason, still grows to fill space
            font=(self.defaultFont.name, 15),
            fg="#5e5f5f",
            wrap="word",
            insertbackground="#5e5f5f",
            selectbackground="#FD6D6D"
        )
        self.message_entry.pack(
            side=tkinter.LEFT,
            fill=tkinter.BOTH,
            expand=True,
            pady=self._entry_padding()
        )
        self.message_entry.insert(1.0, "Type your message")
        self.message_entry.bind("<Control-v>", lambda _: self.master.master.add_attachment("clipboard"))
        self.message_entry.bind("<Return>", self._return)
        self.message_entry.bind("<Key>", self._on_key)  # resizes text box after character is added
        self.message_entry.bind("<BackSpace>", self._delete)  # resizes text box after text is deleted
        self.message_entry.bind("<FocusIn>", self._enter)
        self.message_entry.bind("<FocusOut>", self._leave)

        self._message_scrollbar = ctk.CTkScrollbar(
            self._entry_frame,
            command=self.message_entry.yview,
            bg_color="transparent",
            button_color="#27292A",
            height=0, width=0  # invisible while being packed in order
        )
        self._message_scrollbar.pack(side=tkinter.LEFT)
        self.message_entry.configure(yscrollcommand=self._message_scrollbar.set)

        self._attachments_display = tkinter.Frame(self._entry_frame, bg="#27292A")
        self._attachments_display.pack(side=tkinter.LEFT)

        tkinter.Frame(self._entry_frame, width=14, bg="#27292A").pack(side=tkinter.LEFT)  # spacer

        self.file_button = IconButton(
            self._entry_frame,
            "assets/file.png",
            lambda: self.master.master.add_attachment("file"),
            disabled_icon="assets/file_disabled.png"
        )
        self.file_button.pack(side=ctk.LEFT)

        tkinter.Frame(self._entry_frame, width=14, bg="#27292A").pack(side=tkinter.LEFT)  # spacer

        ctk.CTkFrame(
            self._entry_frame,
            height=32,
            width=4,
            bg_color="#27292A",
            fg_color="#343738"
        ).pack(side=tkinter.LEFT)

        tkinter.Frame(self._entry_frame, width=14, bg="#27292A").pack(side=tkinter.LEFT)  # spacer

        self.send_button = IconButton(
            self._entry_frame,
            "assets/send.png",
            self.master.master.send_message,
            disabled_icon="assets/send_disabled.png"
        )
        self.send_button.pack(side=ctk.LEFT)

        tkinter.Frame(self._entry_frame, width=16, bg="#27292A").pack(side=ctk.LEFT)  # spacer

    def add_message(self, nickname: Optional[str], message: Union[str, Image.Image], clear: bool) -> None:
        if isinstance(message, str):
            self._text_message(nickname, message)
        elif isinstance(message, Image.Image):
            canvas_width = self._canvas.winfo_width()

            # images are displayed with a total of 40px of x padding
            if (message.width + 50) > canvas_width:
                message = resize_image(message, canvas_width - 50)

            # round the corners of the image if supported
            if message.mode != "RGBA":
                mask = Image.new("L", message.size, 0)
                draw = ImageDraw.Draw(mask)
                draw.rounded_rectangle((0, 0, message.width, message.height), 12, fill=255)
                message.putalpha(mask)

            img = ImageTk.PhotoImage(message)

            if nickname:
                self._nickname(nickname)

            frame = tkinter.Frame(self._messages, width=14, bg="#27292A")
            frame.pack(side=tkinter.TOP, anchor=tkinter.W, ipady=5)
            tkinter.Frame(frame, width=20, bg="#27292A").pack(side=tkinter.LEFT)  # spacer

            image = tkinter.Label(
                frame,
                image=img,
                borderwidth=0,
                anchor=tkinter.W,
                cursor="hand2",
                bg="#27292A"
            )

            image.bind("<Button-1>", lambda _: message.show())  # show preview when image is clicked
            image.image = img  # prevents image from being garbage collected
            image.pack(side=tkinter.LEFT)
        else:
            raise ValueError

        if clear:
            self.message_entry.delete(0.0, ctk.END)
            self._resize()

        self._messages.update_idletasks()  # scroll chat frame to bottom
        self._canvas.yview_moveto(1)

    def file_message(self, nickname: str, file: str, real_path: Optional[str] = None) -> None:
        self._nickname(nickname)

        message = tkinter.Text(
            self._messages,
            borderwidth=0,
            bg="#27292A",
            fg="#2588e4",
            font=(self.defaultFont.name, 13),
            highlightthickness=0
        )

        message.insert(1.0, file)
        num_lines, max_line_length = text_size(message)
        message.config(width=max_line_length, height=num_lines)

        message.bind("<Enter>", lambda _: message.configure(font=(self.defaultFont.name, 13, "underline")))
        message.bind("<Leave>", lambda _: message.configure(font=(self.defaultFont.name, 13)))
        message.bind("<Button-1>", lambda _: show_file("downloads/" + file if not real_path else real_path))
        message.configure(cursor="hand2")
        message.pack(side=tkinter.TOP, anchor=tkinter.W, padx=20)

    def get_entry(self) -> Optional[str]:
        message = self.message_entry.get(1.0, ctk.END).strip()

        if message == "" or message == "Type your message":
            return None
        else:
            return message

    def add_attachment(self, file_path: str) -> None:
        self._attachments.append(file_path)

        name, extension = main.parse_file_path(file_path)

        if len(name) > 20:
            name = name[0:20]

        if extension:
            name += f".{extension}"

        frame = tkinter.Frame(self._attachments_display, bg="#27292A")
        frame.pack(anchor=tkinter.E)

        IconButton(
            frame,
            "assets/remove.png",
            command=lambda: self._remove_attachment(name, file_path),
        ).pack(side=tkinter.LEFT)

        tkinter.Label(
            frame,
            text=name,
            bg="#27292A",
            fg="#FFFFFF",
            font=(self.defaultFont.name, 10)
        ).pack(side=tkinter.LEFT)

        debug(f"{file_path} added to attachment list")

    def clear(self) -> None:
        self._clear_children(self._messages)
        tkinter.Frame(self._messages, height=5, bg="#27292A").pack()

    def enable(self) -> None:
        self.send_button.enable()
        self.file_button.enable()

        self.message_entry.delete(0.0, tkinter.END)
        self.message_entry.insert(0.0, "Type your message")
        self.message_entry.configure(
            fg="#5e5f5f",
            font=(self.defaultFont.name, 15),
            cursor="xterm"
        )
        self.message_entry.unbind("<Button-1>")

    def disable(self) -> None:
        self.send_button.disable()
        self.file_button.disable()

        self.message_entry.delete(0.0, tkinter.END)
        self.message_entry.insert(0.0, "Chat disabled")
        self.message_entry.configure(
            fg="#5e5f5f",
            font=(self.defaultFont.name, 15),
            cursor="arrow"
        )
        self.message_entry.pack_configure(pady=self._entry_padding())
        self.message_entry.bind("<Button-1>", lambda _: "break")

    def take_attachments(self) -> List[str]:
        attachments = self._attachments
        self._attachments = []

        self._clear_children(self._attachments_display)
        self._attachments_display.configure(width=1, height=1)  # reset frame to not occupy any space

        return attachments

    # TODO this is still buggy
    def _text_message(self, nickname: Optional[str], body: str) -> None:
        if nickname:
            self._nickname(nickname)

        message = tkinter.Text(
            self._messages,
            borderwidth=0,
            bg="#27292A",
            fg="#c9c9ca",
            font=(self.defaultFont.name, 13 if nickname else 15),
            highlightthickness=0
        )
        message.insert(1.0, body)
        message.pack(side=tkinter.TOP, anchor=tkinter.W, padx=20)

        num_lines, max_line_length = 0, 0

        while True:
            try:
                body, start = self._text_embeds(message, "```")
            except IndexError:
                break

            embed = ctk.CTkFrame(
                message,
                bg_color="transparent",
                fg_color="#191919"
            )

            text = tkinter.Text(
                embed,
                borderwidth=0,
                bg="#191919",
                fg="#c9c9ca",
                highlightthickness=0
            )
            text.insert("1.0", body)

            embed_num_lines, embed_max_line_length = text_size(text)
            text.config(width=embed_max_line_length, height=embed_num_lines)
            max_line_length += embed_max_line_length + 1
            num_lines += embed_num_lines

            text.pack(pady=7, padx=7)
            text.configure(state=tkinter.DISABLED)

            message.window_create(start, window=embed)

        while True:
            try:
                body, start = self._text_embeds(message, "**")
            except IndexError:
                break

            text = tkinter.Text(
                message,
                borderwidth=0,
                bg="#27292A",
                fg="#c9c9ca",
                font=(self.defaultFont.name, 13, "bold"),
                highlightthickness=0
            )
            text.insert("1.0", body)

            embed_num_lines, embed_max_line_length = text_size(text)
            text.config(width=embed_max_line_length, height=embed_num_lines)
            max_line_length += embed_max_line_length

            text.configure(state=tkinter.DISABLED)

            message.window_create(start, window=text)

        while True:
            try:
                body, start = self._text_embeds(message, "*")
            except IndexError:
                break

            text = tkinter.Text(
                message,
                borderwidth=0,
                bg="#27292A",
                fg="#c9c9ca",
                font=(self.defaultFont.name, 13, "italic"),
                highlightthickness=0
            )
            text.insert("1.0", body)

            embed_num_lines, embed_max_line_length = text_size(text)
            text.config(width=embed_max_line_length, height=embed_num_lines)
            max_line_length += embed_max_line_length

            text.configure(state=tkinter.DISABLED)

            message.window_create(start, window=text)

        while True:
            start = message.search(r"\mhttps?://\S+\m", 1.0, regexp=True)

            if not start:
                break

            current_body = message.get(1.0, tkinter.END).strip()
            url = URL_REGEX.search(current_body).group(0)
            end = message.index(f'{start}+{len(url)}c')

            if not end:
                break

            message.delete(start, end)

            text = tkinter.Text(
                message,
                borderwidth=0,
                bg="#27292A",
                fg="#2588e4",
                font=(self.defaultFont.name, 13),
                highlightthickness=0
            )
            text.insert("1.0", url)

            embed_num_lines, embed_max_line_length = text_size(text)
            text.config(width=embed_max_line_length, height=embed_num_lines)
            max_line_length += embed_max_line_length

            text.bind("<Enter>", lambda _: text.configure(font=(self.defaultFont.name, 13, "underline")))
            text.bind("<Leave>", lambda _: text.configure(font=(self.defaultFont.name, 13)))
            text.bind("<Button-1>", lambda _: open_browser(url))
            text.configure(cursor="hand2")
            text.configure(state=tkinter.DISABLED)

            message.window_create(start, window=text)

        a, b = text_size(message)
        message.config(width=max_line_length + b, height=num_lines + a)
        message.configure(state=tkinter.DISABLED)

    # helper for embeds and other text styling things
    @staticmethod
    def _text_embeds(message: tkinter.Text, symbol: str) -> (str, str):
        start = message.search(symbol, 1.0)

        if not start:
            raise IndexError

        end = message.search(symbol, f"{start}+{len(symbol)}c", tkinter.END)

        if not end:
            raise IndexError

        body = message.get(start, f"{end}+{len(symbol)}c").strip(symbol).strip()

        message.delete(start, f"{end}+{len(symbol)}c")

        return body, start

    def _nickname(self, nickname: str) -> None:
        if self._last_nickname == nickname:
            return  # if last message had same nickname no need to display again
        else:
            self._last_nickname = nickname

        label = tkinter.Label(
            self._messages,
            text=nickname,
            bg="#27292A",
            fg="#FFFFFF",
            font=(self.defaultFont.name, 15),
            anchor=tkinter.W
        )

        label.pack(anchor=tkinter.W, padx=20)

    def _remove_attachment(self, name: str, file_path: str) -> None:
        self._attachments.remove(file_path)  # remove file path from list

        # delete attachment from display
        for child in self._attachments_display.winfo_children():
            if child.winfo_children()[1].cget("text") == name:
                child.destroy()

        if len(self._attachments_display.winfo_children()) == 0:  # if no attachments left
            self._attachments_display.configure(width=1, height=1)  # reset frame to not occupy any space

    def _return(self, event: tkinter.Event) -> str:
        if platform == "win32":
            if event.state == 8:  # no modifier key is pressed
                self.master.master.send_message()
            elif event.state == 9:  # shift key is pressed
                self.message_entry.insert(ctk.INSERT, "\n")
                self._on_key(event)

        elif platform == "darwin":
            if event.state == 0:  # no modifier key is pressed
                self.master.master.send_message()
            elif event.state == 1:  # shift key is pressed
                self.message_entry.insert(ctk.INSERT, "\n")
                self._on_key(event)

        elif platform == "linux":
            if event.state == 16:  # no modifier key is pressed
                self.master.master.send_message()
            elif event.state == 17:  # shift key is pressed
                self.message_entry.insert(ctk.INSERT, "\n")
                self._on_key(event)

        return "break"

    def _delete(self, _) -> None:
        self.message_entry.after(0, self._resize)  # resize after deletion

    def _on_key(self, _) -> None:
        self.message_entry.after(0, self._resize)  # resize after character added

    def _resize(self):
        num_lines = int(self.message_entry.index("end-1c").split(".")[0])

        if self.message_entry.yview()[0] > 0:  # check if message entry needs scrollbar
            self._message_scrollbar.configure(height=55, width=16, button_color="gray41")
            self.message_entry.yview_moveto(1)  # scroll to bottom
        else:
            self.message_entry.configure(height=num_lines)  # increase display lines until no more space
            self.message_entry.update_idletasks()
            self._message_scrollbar.configure(height=0, width=0, button_color="#27292A")

        self.message_entry.pack_configure(pady=max(20 - (3 * (num_lines - 1)), 10))

    def _enter(self, _) -> None:  # focus in event handler
        if self.message_entry.get(1.0, tkinter.END).strip() == "Type your message":  # if placeholder is in clear it
            self.message_entry.delete(0.0, tkinter.END)
            self.message_entry.configure(fg="#FFFFFF", font=(self.defaultFont.name, 13))
            self.message_entry.pack_configure(pady=20)

    def _leave(self, _) -> None:  # focus out event handler
        if self.message_entry.get(1.0, tkinter.END).strip() == "":  # if input is empty put the placeholder in
            self.message_entry.insert(1.0, "Type your message")
            self.message_entry.configure(fg="#5e5f5f", font=(self.defaultFont.name, 15))
            self.message_entry.pack_configure(pady=self._entry_padding())

    @staticmethod
    def _clear_children(widget: any) -> None:
        for child in widget.winfo_children():
            try:
                child.destroy()
            except tkinter.TclError:
                continue

    @staticmethod
    def _entry_padding() -> int:
        if platform == "win32":
            return 18
        else:
            return 19


class SettingsWidget(ctk.CTkToplevel):
    def __init__(self, master: any, chat_config: main.Config, p: PyAudio, **kwargs):
        super().__init__(master, **kwargs)

        self.chat_config = chat_config

        self.configure(fg_color="#222425")
        self.geometry("620x400")
        self.title("Settings")
        self.resizable(False, False)

        if platform == "win32":
            self.iconbitmap("assets/settings.ico")

        self.wm_transient(master)

        if platform != "linux":
            self.grab_set()

        self.defaultFont = self.master.defaultFont

        try:
            self.input_devices = main.get_devices(p, "maxInputChannels")
        except IndexError:
            self.input_devices = []

        try:
            self.output_devices = main.get_devices(p, "maxOutputChannels")
        except IndexError:
            self.output_devices = []

        # TODO improve this layout

        tkinter.Frame(self, height=10, bg="#222425").pack()  # spacer

        control_frame = tkinter.Frame(self, bg="#222425")
        control_frame.pack(pady=7)

        ctk.CTkLabel(control_frame, text="Control Port").pack(side=tkinter.LEFT)

        tkinter.Frame(control_frame, width=30, bg="#222425").pack(side=tkinter.LEFT)  # spacer

        control_variable = ctk.StringVar(value=str(self.chat_config.controlPort))
        control_entry = ctk.CTkEntry(
            control_frame,
            width=70,
            textvariable=control_variable
        )
        control_entry.bind("<KeyRelease>", lambda _: self._control_port_changed(control_variable))
        control_entry.pack(side=tkinter.LEFT)

        audio_frame = tkinter.Frame(self, bg="#222425")
        audio_frame.pack(pady=7)

        ctk.CTkLabel(audio_frame, text="Audio Ports").pack(side=tkinter.LEFT)

        tkinter.Frame(audio_frame, width=30, bg="#222425").pack(side=tkinter.LEFT)  # spacer

        actual_range = PortRange(self.chat_config.audioPorts)
        start_variable = ctk.StringVar(value=str(actual_range.port_from))
        end_variable = ctk.StringVar(value=str(actual_range.port_to))

        start_entry = ctk.CTkEntry(
            audio_frame,
            width=70,
            textvariable=start_variable
        )
        start_entry.bind("<KeyRelease>", lambda _: self._audio_port_changed(start_variable, end_variable))
        start_entry.pack(side=tkinter.LEFT)

        tkinter.Frame(audio_frame, width=5, bg="#222425").pack(side=tkinter.LEFT)  # spacer
        ctk.CTkLabel(audio_frame, text="to").pack(side=tkinter.LEFT)
        tkinter.Frame(audio_frame, width=7, bg="#222425").pack(side=tkinter.LEFT)  # spacer

        end_entry = ctk.CTkEntry(
            audio_frame,
            width=70,
            textvariable=end_variable
        )
        end_entry.bind("<KeyRelease>", lambda _: self._audio_port_changed(start_variable, end_variable))
        end_entry.pack(side=tkinter.LEFT)

        input_frame = tkinter.Frame(self, bg="#222425")
        input_frame.pack(pady=7)

        ctk.CTkLabel(input_frame, text="Input Device").pack(side=tkinter.LEFT)

        tkinter.Frame(input_frame, width=40, bg="#222425").pack(side=tkinter.LEFT)  # spacer

        ctk.CTkComboBox(
            input_frame,
            values=[device.name for device in self.input_devices],
            command=self._input_changed,
            width=300,
            variable=ctk.StringVar(value=self.chat_config.inputDevice)
        ).pack(side=tkinter.LEFT)

        output_frame = tkinter.Frame(self, bg="#222425")
        output_frame.pack(pady=7)

        ctk.CTkLabel(output_frame, text="Output Device").pack(side=tkinter.LEFT)

        tkinter.Frame(output_frame, width=30, bg="#222425").pack(side=tkinter.LEFT)  # spacer

        ctk.CTkComboBox(
            output_frame,
            values=[device.name for device in self.output_devices],
            command=self._output_changed,
            width=300,
            variable=ctk.StringVar(value=self.chat_config.outputDevice)
        ).pack(side=tkinter.LEFT)

        # rnnoise is only available on Windows currently
        if platform == "win32":
            ctk.CTkSwitch(
                self,
                text="Use RNNoise for noise suppression",
                command=self._suppression_changed,
                variable=ctk.IntVar(value=1 if master.chat_config.useRnnoise else 0)
            ).pack(pady=10)

            options_frame = tkinter.Frame(self, bg="#222425")
            options_frame.pack()

            vt_frame = tkinter.Frame(options_frame, bg="#222425")
            vt_frame.pack(side=tkinter.LEFT)
            tkinter.Frame(options_frame, width=20, bg="#222425").pack(side=tkinter.LEFT)  # spacer

            ctk.CTkLabel(vt_frame, text="VAD Threshold").pack()

            slider = ctk.CTkSlider(
                vt_frame,
                to=1,
                from_=0,
                width=100,
                command=self._update_vt
            )
            slider.set(self.chat_config.VADThreshold)
            slider.pack(pady=5)

            self.vt_display = ctk.CTkLabel(
                vt_frame,
                text=str(round(self.chat_config.VADThreshold, 2))
            )
            self.vt_display.pack()

            vg_frame = tkinter.Frame(options_frame, bg="#222425")
            vg_frame.pack(side=tkinter.LEFT)
            tkinter.Frame(options_frame, width=20, bg="#222425").pack(side=tkinter.LEFT)  # spacer

            ctk.CTkLabel(vg_frame, text="VAD Grace Period").pack()

            slider = ctk.CTkSlider(
                vg_frame,
                to=1,
                from_=0,
                width=100,
                command=self._update_vg
            )
            slider.set(self.chat_config.VADGracePeriod)
            slider.pack(pady=5)

            self.vg_display = ctk.CTkLabel(
                vg_frame,
                text=str(round(self.chat_config.VADGracePeriod, 2))
            )
            self.vg_display.pack()

            rv_frame = tkinter.Frame(options_frame, bg="#222425")
            rv_frame.pack(side=tkinter.LEFT)

            ctk.CTkLabel(rv_frame, text="Retroactive VAD Grace Period").pack()

            slider = ctk.CTkSlider(
                rv_frame,
                to=1,
                from_=0,
                width=100,
                command=self._update_rv
            )
            slider.set(self.chat_config.retroactiveVADGracePeriod)
            slider.pack(pady=5)

            self.rv_display = ctk.CTkLabel(
                rv_frame,
                text=str(round(self.chat_config.retroactiveVADGracePeriod, 2))
            )
            self.rv_display.pack()

        self.test_button = ctk.CTkButton(self, text="Start Audio Test", command=self._start_test)
        self.test_button.pack(pady=10)

        ctk.CTkButton(self, text="Reset defaults", command=self._reset_defaults).pack(pady=10)

    # start audio test
    def _start_test(self):
        # change button to end audio test
        self.test_button.configure(text="End Audio Test")
        self.test_button.configure(command=self._end_test)
        self.master.audio_test()  # start audio test

    # end audio test
    def _end_test(self):
        # change button to start audio test
        self.test_button.configure(text="Start Audio Test")
        self.test_button.configure(command=self._start_test)
        self.master.end_call(False, False, True)  # end audio test

    # callback for input device change
    def _input_changed(self, value: str) -> None:
        for device in self.input_devices:
            if device.name == value:
                self.chat_config.inputDevice = value
                break

        if self.master.in_call:
            # reconfigure stream if in call
            self.master.configure_stream()

    # callback for output device change
    def _output_changed(self, value: str) -> None:
        for device in self.output_devices:
            if device.name == value:
                self.chat_config.outputDevice = value
                break

        if self.master.in_call:
            # reconfigure stream if in call
            self.master.configure_stream()

    # callback for noise suppression change
    def _suppression_changed(self):
        self.chat_config.useRnnoise = not self.chat_config.useRnnoise

    def _update_vt(self, value: float):
        rounded = round(value, 2)
        self.vt_display.configure(text=str(rounded))
        self.chat_config.VADThreshold = rounded

    def _update_vg(self, value: float):
        rounded = round(value, 2)
        self.vg_display.configure(text=str(rounded))
        self.chat_config.VADGracePeriod = rounded

    def _update_rv(self, value: float):
        rounded = round(value, 2)
        self.rv_display.configure(text=str(rounded))
        self.chat_config.retroactiveVADGracePeriod = rounded

    def _reset_defaults(self):
        self.master.chat_config.default()
        self.destroy()

    def _control_port_changed(self, control_variable: ctk.StringVar):
        self.chat_config.controlPort = int(control_variable.get())

    def _audio_port_changed(self, start_variable: ctk.StringVar, end_variable: ctk.StringVar):
        self.chat_config.audioPort = f"{start_variable.get()}-{end_variable.get()}"


class ScreenshareWidget(ctk.CTkToplevel):
    _input: str
    _output_resolution: str
    _bitrate: int
    _fps: int

    def __init__(self, master: main.App, *args, **kwargs):
        super().__init__(*args, **kwargs)

        # default values
        self._input = "Fullscreen"
        self._output_resolution = "720p"
        self._bitrate = 8000
        self._fps = 30

        self.geometry("620x325")
        self.title("Screenshare Configurator")
        self.resizable(False, False)

        if platform == "win32":
            self.iconbitmap("assets/settings.ico")

        self.geometry(f"+{master.winfo_x() + 100}+{master.winfo_y() + 100}")
        self.wm_transient(master)

        tkinter.Frame(self, height=20, bg="#191919").pack()  # spacer

        # only windows supports per window capture
        if platform == "win32":
            ctk.CTkLabel(self, text="Screenshare Option").pack()

            finder = MainWindowHandlesFinder()
            windows = [window[1] for window in finder.find_main_window_handles()]
            windows.append("Fullscreen")
            windows.reverse()

            ctk.CTkComboBox(
                self,
                values=windows,
                command=self._input_changed,
                width=300,
                variable=ctk.StringVar(value=self._input)
            ).pack()

        ctk.CTkLabel(self, text="Output Resolution").pack()

        ctk.CTkComboBox(
            self,
            values=["1080p", "720p", "480p"],
            command=self._output_resolution_changed,
            width=300,
            variable=ctk.StringVar(value=self._output_resolution)
        ).pack()

        tkinter.Frame(self, height=15, bg="#191919").pack()  # spacer

        self.bitrate_display = ctk.CTkLabel(
            self,
            text=f"Bitrate: {self._bitrate} kbps"
        )
        self.bitrate_display.pack()

        ctk.CTkSlider(
            self,
            to=30000,
            from_=1000,
            command=self._update_bitrate,
            variable=ctk.IntVar(value=self._bitrate)
        ).pack(pady=5)

        self.fps_display = ctk.CTkLabel(
            self,
            text=f"FPS: {self._fps}"
        )
        self.fps_display.pack()

        ctk.CTkSlider(
            self,
            to=60,
            from_=1,
            command=self._update_fps,
            variable=ctk.IntVar(value=self._fps)
        ).pack()

        tkinter.Frame(self, height=20, bg="#191919").pack()  # spacer

        ctk.CTkButton(self, text="Start stream", command=self.start).pack(pady=10)

    # start stream
    def start(self):
        if self.master.audio_socket is None:  # no call
            return

        other_ip = self.master.audio_socket.getpeername()[0]
        contact = self.master.connected_contact

        salt = secrets.token_bytes(14)  # random salt
        port = contact.screenshare_handshake(salt)  # perform handshake, includes sending sdp
        key = main.b64_encode(contact.secret.encode() + salt)  # key for ffmpeg
        command = self._format_command(other_ip, port, key)  # format the ffmpeg command

        self.master.in_screenshare = True
        # change button to end screenshare
        self.master.screenshare_button.change_icon("assets/end_screenshare.png")
        self.master.screenshare_button.change_command(self.master.end_screenshare)

        # start ffmpeg in a new thread
        Thread(target=self.master.record_screenshare, args=(command,)).start()
        self.destroy()  # close the window/widget

    # callback for input device combobox
    def _input_changed(self, value: str):
        self._input = value

    # callback for output resolution combobox
    def _output_resolution_changed(self, value: str):
        self._output_resolution = value

    # callback for bitrate slider
    def _update_bitrate(self, value: float):
        self._bitrate = int(value)
        self.bitrate_display.configure(text=f"Bitrate: {self._bitrate} kbps")

    # callback for fps slider
    def _update_fps(self, value: float):
        self._fps = int(value)
        self.fps_display.configure(text=f"FPS: {self._fps}")

    # format the ffmpeg command
    def _format_command(self, target_host: str, port: int, key: str) -> str:  # TODO test linux support
        if self._input == "Fullscreen":
            if platform == "darwin":
                i = "\"default:none\""
            elif platform == "win32":
                i = "desktop"
            elif platform == "linux":
                i = ":0.0"
            else:
                raise NotImplementedError
        else:  # only windows supports window capture
            # TODO set window as active so ffplay can always capture it
            i = f"title=\"{self._input}\""

        if platform == "darwin":
            capture_device = "avfoundation"
        elif platform == "win32":
            capture_device = "gdigrab"
        elif platform == "linux":
            capture_device = "x11grab"
        else:
            raise NotImplementedError

        r = self._output_resolution.rstrip("p")

        return f"ffmpeg -f {capture_device} -framerate {self._fps} -i {i} -preset ultrafast -tune " \
               f"zerolatency -tune fastdecode -movflags +faststart -c:v libx264 -b:v {self._bitrate}k " \
               f"-vf scale=\"trunc(oh*a/2)*2:{r}\" -x264opts \"slice-max-size=10000:sync-lookahead=0\" -g 30 -f rtp " \
               f"-x264-params \"nal-hrd=cbr:force-cfr=1\" -srtp_out_suite AES_CM_128_HMAC_SHA1_80 " \
               f"-srtp_out_params {key} srtp://{target_host}:{port}"


# show file in file explorer
def show_file(file_path: str) -> None:
    if platform == "win32":  # this code is more reliable than a version that selects the file in explorer
        # windows only imports
        from os import startfile
        from os.path import dirname

        folder_path = dirname(file_path)
        startfile(folder_path)
        # Popen(["explorer", "/select,", file_path]) this code selects the file in explorer, but is less reliable
    elif platform == "darwin":
        Popen(["open", "-R", file_path])
    elif platform == "linux":
        # linux only import
        from os.path import dirname

        folder_path = dirname(file_path)
        Popen(["xdg-open", folder_path])


# open url in browser
def open_browser(url: str) -> None:
    debug(f"opening {url}")
    webbrowser.open(url, new=2, autoraise=True)


# get the size of a tkinter text widget
def text_size(text: tkinter.Text) -> Tuple[int, int]:
    num_lines = int(text.index("end-1c").split('.')[0])
    max_line_length = max([len(text.get(f"{i}.0", f"{i}.end")) for i in range(1, num_lines + 1)])
    return num_lines, max_line_length


# resize an image to a certain width while minting aspect ratio
def resize_image(image: Image.Image, width: int) -> Image.Image:
    current_width, current_height = image.size
    aspect_ratio = current_height / current_width
    new_height = int(aspect_ratio * width)
    resized_image = image.resize((width, new_height))
    return resized_image


if platform == "win32":
    # code from https://github.com/Kalmat/PyWinCtl
    # refactored to fit standard naming conventions and simplify dependencies
    class MainWindowHandlesFinder:
        # define a ctypes structure to retrieve title bar info
        class TitleBarInfo(ctypes.Structure):
            _fields_ = [
                ("cbSize", wintypes.DWORD),  # size of the structure
                ("rcTitleBar", wintypes.RECT),  # rectangle coordinates of the title bar
                ("rgstate", wintypes.DWORD * 6)  # array describing the state of the title bar
            ]

        def __init__(self) -> None:
            # initialize an empty list for storing window handles and titles
            self.handle_list: List[Tuple[int, str]] = []

        def win_enum_handler(self, hwnd: int, _) -> None:
            # if the window is not visible, return immediately
            if not win32gui.IsWindowVisible(hwnd):
                return

            # create an instance of title bar info and set its size
            title_info = self.TitleBarInfo()
            title_info.cbSize = ctypes.sizeof(title_info)

            # call the windows api function to get title bar info
            ctypes.windll.user32.GetTitleBarInfo(hwnd, ctypes.byref(title_info))
            # initialize a variable to store whether the window is cloaked
            is_cloaked = ctypes.c_int(0)
            # call the windows api function to check if the window is cloaked
            ctypes.windll.dwmapi.DwmGetWindowAttribute(hwnd, 14, ctypes.byref(is_cloaked), ctypes.sizeof(is_cloaked))

            # get the window's title
            title = win32gui.GetWindowText(hwnd)

            # if the window is not cloaked and has a title
            if title and is_cloaked.value == 0:
                # if the title bar is not invisible, add the window handle and title to the list
                if not (title_info.rgstate[0] & win32con.STATE_SYSTEM_INVISIBLE):
                    self.handle_list.append((hwnd, title))

        def find_main_window_handles(self) -> List[Tuple[int, str]]:
            # enumerate all windows and call the callback function for each one
            win32gui.EnumWindows(self.win_enum_handler, None)

            # return the list of window handles and titles
            return self.handle_list


# TODO works but is terrible
# function to count the number of *displayed* lines in a Text widget
"""
def count_display_lines(text_widget: tkinter.Text) -> int:  
    # create a hidden Text widget with the same configuration
    frame = tkinter.Frame(text_widget.master, width=text_widget.winfo_width(), height=100)
    frame.pack()
    # frame.place(x=-1000, y=-1000)  # Move the hidden widget off-screen
    frame.update_idletasks()

    hidden_text = tkinter.Text(
        frame,
        wrap="word",
        font=text_widget["font"]
    )
    hidden_text.pack(expand=True, fill="both")
    hidden_text.update_idletasks()
    debug(hidden_text.winfo_width())

    content = text_widget.get("1.0", "end-1c")
    hidden_text.insert("1.0", content)

    line_count = 0
    current_index = "1.0"

    while True:
        # get the line information for the current index
        line_info = hidden_text.dlineinfo(current_index)

        # line_info for end is None
        if line_info:
            line_count += 1
        else:
            break

        # move the current index to the next displayed line
        current_index = hidden_text.index(f"{current_index} + 1 display line")

    hidden_text.destroy()
    frame.destroy()

    return line_count"""
