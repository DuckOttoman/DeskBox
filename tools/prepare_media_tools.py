"""Copy an installed imageio-ffmpeg runtime into the portable application."""
from pathlib import Path
import shutil, subprocess
import imageio_ffmpeg
root=Path(__file__).resolve().parents[1]
source=Path(imageio_ffmpeg.get_ffmpeg_exe())
for folder in [root/'target/release/media-tools',root/'bin/media-tools']:
    folder.mkdir(parents=True,exist_ok=True)
    shutil.copy2(source,folder/'ffmpeg.exe')
    for flag,name in [('-L','FFmpeg-LICENSE.txt'),('-version','FFmpeg-VERSION.txt')]:
        result=subprocess.run([str(source),flag],capture_output=True,text=True)
        (folder/name).write_text(result.stdout+result.stderr+'\nUpstream: https://ffmpeg.org/\nProvider: https://github.com/imageio/imageio-ffmpeg\n',encoding='utf-8')
print('FFmpeg runtime prepared; retain its license/version notices with distribution.')
