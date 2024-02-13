import QtQuick
import QtQuick.Layouts
import QtQuick.Window
import Rx.Widgets

Item {
    id: root

    property string type
    property string remoteAddr
    property string tcpAddr
    property string wsAddr
    property int latency

    height: 70

    Rectangle {
        id: activeRoot
        width: 850
        height: 70
        color: hoverHandler.hovered ? Style.palette.button : "transparent"
        Behavior on color {
            ColorAnimation {
                duration: Style.midAnimationDuration
            }
        }
        HoverHandler {
            id: hoverHandler
        }
        Rectangle {
            id: activeClients
            anchors.centerIn: parent
            anchors.topMargin: 5
            width: 830
            height: 50
            color: "transparent"
            
            Row {
                Rectangle {
                    width: 738
                    height: parent.height
                    color: "transparent"
                    Column {
                        spacing: 8
                        Row {
                            spacing: 30
                            Label {
                                text: remoteAddr + " -> " + tcpAddr
                            }
                            Label {
                                text: latency + " ms"
                                color: type == "active" ? "green" : "yellow"
                            }
                        }
                        Label {
                            text: wsAddr
                            opacity: 0.6
                        }
                    }
                }
                

                Row {
                    Button {
                        id: copyButton
                        display: AbstractButton.IconOnly
                        borderWidth: 0
                        flat: true
                        icon.source: "qrc:/resources/assets/copy.svg"
                        icon.width: 20
                        icon.height: 20
                        icon.color: "green"
                        opacity: hoverHandler.hovered ? 1 : 0
                        onClicked: {
                        }
                        Behavior on opacity {
                            NumberAnimation {
                                duration: Style.midAnimationDuration
                            }
                        }
                        ToolTip {
                            parent: copyButton
                            visible: parent.hovered
                            text: qsTr("Copy local address")
                        }
                    }
                    Button {
                        id: dismissButtion
                        display: AbstractButton.IconOnly
                        borderWidth: 0
                        flat: true
                        icon.source: "qrc:/resources/assets/dismiss.svg"
                        icon.width: 20
                        icon.height: 20
                        opacity: hoverHandler.hovered ? 1 : 0
                        onClicked: {
                            api.cancelClient(remoteAddr, wsAddr, tcpAddr, type)
                        }
                        Behavior on opacity {
                            NumberAnimation {
                                duration: Style.midAnimationDuration
                            }
                        }
                        ToolTip {
                            parent: dismissButtion
                            visible: parent.hovered

                            text: qsTr("Delete connection")
                        }
                    }
                }
            }
        }
        Rectangle {
            anchors.left: parent.left
            anchors.top: parent.bottom
            width: parent.width
            height: 1
            color: Style.palette.button
        }
    }
}