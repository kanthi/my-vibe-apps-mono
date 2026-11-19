/* Slideshow shown while Tanix is being installed. */

import QtQuick 2.0;
import calamares.slideshow 1.0;

Presentation
{
    id: presentation

    function nextSlide() {
        presentation.goToNextSlide();
    }

    Timer {
        id: advanceTimer
        interval: 20000
        running: presentation.activatedInCalamares
        repeat: true
        onTriggered: nextSlide()
    }

    Slide {
        Text {
            id: title1
            anchors.centerIn: parent
            text: "Welcome to Tanix"
            font.pixelSize: 30
            width: presentation.width
            horizontalAlignment: Text.Center
            color: "#2B4FC0"
        }
        Image {
            id: image1
            source: "tanix-logo.png"
            width: 180; height: 180
            fillMode: Image.PreserveAspectFit
            anchors.bottom: title1.top
            anchors.horizontalCenter: parent.horizontalCenter
        }
        Text {
            anchors.horizontalCenter: image1.horizontalCenter
            anchors.top: title1.bottom
            text: "Your system is being built from a single configuration file.<br/>"+
                  "The software for the system you picked is downloaded now,<br/>"+
                  "so this step <b>needs an internet connection</b><br/>"+
                  "and can take a while."
            wrapMode: Text.WordWrap
            width: presentation.width
            horizontalAlignment: Text.Center
        }
    }

    Slide {
        Text {
            id: title2
            anchors.centerIn: parent
            text: "Built on NixOS"
            font.pixelSize: 30
            width: presentation.width
            horizontalAlignment: Text.Center
            color: "#2B4FC0"
        }
        Image {
            id: image2
            source: "tanix-logo.png"
            width: 180; height: 180
            fillMode: Image.PreserveAspectFit
            anchors.bottom: title2.top
            anchors.horizontalCenter: parent.horizontalCenter
        }
        Text {
            anchors.horizontalCenter: image2.horizontalCenter
            anchors.top: title2.bottom
            text: "Every change to the system is a new generation.<br/>"+
                  "If an update goes wrong, <b>pick the previous generation<br/>"+
                  "in the boot menu</b> and you are back where you were."
            wrapMode: Text.WordWrap
            width: presentation.width
            horizontalAlignment: Text.Center
        }
    }

    Slide {
        Text {
            id: title3
            anchors.centerIn: parent
            text: "Change it later"
            font.pixelSize: 30
            width: presentation.width
            horizontalAlignment: Text.Center
            color: "#2B4FC0"
        }
        Image {
            id: image3
            source: "tanix-logo.png"
            width: 180; height: 180
            fillMode: Image.PreserveAspectFit
            anchors.bottom: title3.top
            anchors.horizontalCenter: parent.horizontalCenter
        }
        Text {
            anchors.horizontalCenter: image3.horizontalCenter
            anchors.top: title3.bottom
            text: "The installed system is described in<br/>"+
                  "<b>/etc/nixos/configuration.nix</b>. Edit it and run<br/>"+
                  "<b>sudo nixos-rebuild switch</b> to add software<br/>"+
                  "or switch to another desktop."
            wrapMode: Text.WordWrap
            width: presentation.width
            horizontalAlignment: Text.Center
        }
    }

    function onActivate() {
        presentation.currentSlide = 0;
    }

    function onLeave() {
    }
}
