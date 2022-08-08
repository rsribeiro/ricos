# VirtualBox
qemu-img convert -f raw -O qcow2 `
target/x86_64-blog_os/debug/bootimage-blog_os.bin `
build/image.qcow2

# Bochs
& 'bximage.exe' -func=convert -imgmode=flat -q `
target/x86_64-blog_os/debug/bootimage-blog_os.bin `
build/image.img

# Release
& 'bximage.exe' -func=convert -imgmode=flat -q `
target/x86_64-blog_os/release/bootimage-blog_os.bin `
build/RicOS.img
