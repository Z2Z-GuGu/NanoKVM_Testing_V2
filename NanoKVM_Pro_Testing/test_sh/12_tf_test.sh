#!/bin/bash

SAVE_DIR="/root/factory_test/done"

tf_test() {
    if ls /dev/mmcblk1* >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

if [ ! -f "$SAVE_DIR/.tf.done" ]; then
    if tf_test $1; then
        touch "$SAVE_DIR/.tf.done"
        echo "TF test passed"
    else
        echo "TF test failed"
    fi
else
    echo "TF test passed"
fi

sync

echo "Finish"

