#!/bin/sh

cd "$(dirname "$0")"
rm com/moulberry/pandora/LaunchWrapper.class
javac com/moulberry/pandora/LaunchWrapper.java
jar cvf LaunchWrapper.jar com/moulberry/pandora/LaunchWrapper.class
