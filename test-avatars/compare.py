import subprocess
import os
import sys
for f in os.listdir("."):
    if not f.endswith(".png"):
        continue

    s = subprocess.check_output(["dct-tiv", f, "--spatial"])[:-1]
    d = subprocess.check_output(["dct-tiv", f, "--dct"])[:-1]
    t = subprocess.check_output(["tiv", "-w", "2", "-h", "1", f])[:-1]
    sys.stdout.buffer.write(s)
    sys.stdout.buffer.write(b"  ")
    sys.stdout.buffer.write(d)
    sys.stdout.buffer.write(b"  ")
    sys.stdout.buffer.write(t)
    sys.stdout.buffer.write(f" \n\n".encode("utf-8"))
    sys.stdout.flush()
    subprocess.run(["./display_image.sh", f])
