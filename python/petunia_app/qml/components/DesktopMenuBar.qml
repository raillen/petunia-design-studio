import QtQuick
import QtQuick.Controls

MenuBar {
    id: desktopMenuBar

    signal requestNewDocument()
    signal requestDocumentSetup()
    signal requestAppSettings()
    signal requestHelp()
    signal requestPlaceImage()

    background: Rectangle {
        color: "#18181a"
        border.color: "#27272a"
        border.width: 1
    }

    delegate: MenuBarItem {
        id: mbItem
        contentItem: Text {
            text: mbItem.text.replace(/&/g, "")
            font.pixelSize: 11
            color: mbItem.highlighted ? "#ffffff" : "#a1a1aa"
            horizontalAlignment: Text.AlignLeft
            verticalAlignment: Text.AlignVCenter
        }
        background: Rectangle {
            color: mbItem.highlighted ? "#2e2e33" : "transparent"
            radius: 3
        }
    }

    // -------------------------------------------------------------
    // Menu Arquivo
    // -------------------------------------------------------------
    Menu {
        title: qsTr("&Arquivo")

        Action {
            text: qsTr("&Novo Documento...")
            shortcut: StandardKey.New
            onTriggered: desktopMenuBar.requestNewDocument()
        }
        Action {
            text: qsTr("&Abrir Documento PTND...")
            shortcut: StandardKey.Open
            onTriggered: bridge.open("/tmp/document.ptnd")
        }
        Action {
            text: qsTr("&Salvar")
            shortcut: StandardKey.Save
            onTriggered: bridge.save("/tmp/document.ptnd")
        }
        Action {
            text: qsTr("Salvar &Como...")
            shortcut: StandardKey.SaveAs
            onTriggered: bridge.save("/tmp/document.ptnd")
        }

        MenuSeparator {}

        Action {
            text: qsTr("&Inserir Imagem...")
            onTriggered: desktopMenuBar.requestPlaceImage()
        }
        Action {
            text: qsTr("Inserir &Prancheta Preset")
            onTriggered: bridge.insertPresetArtboard("doc")
        }

        MenuSeparator {}

        Action {
            text: qsTr("&Exportar Imagem PNG...")
            onTriggered: bridge.exportPng("/tmp/petunia-design-export.png")
        }
        Action {
            text: qsTr("Exportar Vetor &SVG...")
            onTriggered: bridge.exportSvg()
        }

        MenuSeparator {}

        Action {
            text: qsTr("Configuração do &Documento...")
            onTriggered: desktopMenuBar.requestDocumentSetup()
        }

        MenuSeparator {}

        Action {
            text: qsTr("&Sair")
            shortcut: StandardKey.Quit
            onTriggered: Qt.quit()
        }
    }

    // -------------------------------------------------------------
    // Menu Editar
    // -------------------------------------------------------------
    Menu {
        title: qsTr("&Editar")

        Action {
            text: qsTr("&Desfazer")
            shortcut: StandardKey.Undo
            enabled: bridge.canUndo
            onTriggered: bridge.undo()
        }
        Action {
            text: qsTr("&Refazer")
            shortcut: StandardKey.Redo
            enabled: bridge.canRedo
            onTriggered: bridge.redo()
        }

        MenuSeparator {}

        Action {
            text: qsTr("&Duplicar Seleção")
            shortcut: "Ctrl+D"
            enabled: bridge.selectedIds.length > 0
            onTriggered: bridge.duplicateSelected()
        }
        Action {
            text: qsTr("&Excluir")
            shortcut: StandardKey.Delete
            enabled: bridge.selectedIds.length > 0
            onTriggered: bridge.deleteSelected()
        }

        MenuSeparator {}

        Action {
            text: qsTr("Alternar &Alinhamento Magnético (Snap)")
            shortcut: "Ctrl+;"
            onTriggered: bridge.toggleSnapping()
        }

        Action {
            text: qsTr("&Preferências do Aplicativo...")
            shortcut: "Ctrl+,"
            onTriggered: desktopMenuBar.requestAppSettings()
        }
    }

    // -------------------------------------------------------------
    // Menu Visualizar
    // -------------------------------------------------------------
    Menu {
        title: qsTr("&Visualizar")

        Action {
            text: qsTr("Aumentar Zoom (&+)")
            shortcut: "Ctrl+="
            onTriggered: bridge.zoomIn()
        }
        Action {
            text: qsTr("Diminuir Zoom (&-)")
            shortcut: "Ctrl+-"
            onTriggered: bridge.zoomOut()
        }
        Action {
            text: qsTr("&Ajustar Pranchetas à Tela")
            shortcut: "Ctrl+0"
            onTriggered: bridge.fitAllArtboards()
        }
        Action {
            text: qsTr("Tamanho Real (&100%)")
            shortcut: "Ctrl+1"
            onTriggered: bridge.setZoom(1.0)
        }

        MenuSeparator {}

        Action {
            text: qsTr("Persona &Vetor")
            onTriggered: {
                bridge.setPersona("vector")
                bridge.setTool("select")
            }
        }
        Action {
            text: qsTr("Persona &Pixel")
            onTriggered: {
                bridge.setPersona("pixel")
                bridge.setTool("pixel_brush")
            }
        }
        Action {
            text: qsTr("Persona &Exportação")
            onTriggered: {
                bridge.setPersona("export")
                bridge.setTool("slice")
            }
        }
    }

    // -------------------------------------------------------------
    // Menu Camada
    // -------------------------------------------------------------
    Menu {
        title: qsTr("&Camada")

        Action {
            text: qsTr("&Agrupar Objetos")
            shortcut: "Ctrl+G"
            enabled: bridge.selectedIds.length > 1
            onTriggered: bridge.groupSelected()
        }
        Action {
            text: qsTr("&Desagrupar Objeto")
            shortcut: "Ctrl+Shift+G"
            enabled: bridge.selectedIds.length > 0
            onTriggered: bridge.ungroupSelected()
        }

        MenuSeparator {}

        Action {
            text: qsTr("&Converter em Curvas")
            shortcut: "Ctrl+Return"
            enabled: bridge.selectedId !== ""
            onTriggered: bridge.convertToCurves(bridge.selectedId)
        }

        MenuSeparator {}

        Action {
            text: qsTr("Alinhar à &Esquerda")
            onTriggered: bridge.alignSelected("left")
        }
        Action {
            text: qsTr("Alinhar ao &Centro")
            onTriggered: bridge.alignSelected("center")
        }
        Action {
            text: qsTr("Alinhar à &Direita")
            onTriggered: bridge.alignSelected("right")
        }
        Action {
            text: qsTr("Alinhar ao &Topo")
            onTriggered: bridge.alignSelected("top")
        }
        Action {
            text: qsTr("Alinhar ao &Meio")
            onTriggered: bridge.alignSelected("middle")
        }
        Action {
            text: qsTr("Alinhar ao &Fundo")
            onTriggered: bridge.alignSelected("bottom")
        }
    }

    // -------------------------------------------------------------
    // Menu Geometria
    // -------------------------------------------------------------
    Menu {
        title: qsTr("&Geometria")

        Action {
            text: qsTr("&Adicionar (União Booleana)")
            enabled: bridge.selectedIds.length > 1
            onTriggered: bridge.boolean("union")
        }
        Action {
            text: qsTr("&Subtrair")
            enabled: bridge.selectedIds.length > 1
            onTriggered: bridge.boolean("difference")
        }
        Action {
            text: qsTr("&Intersecção")
            enabled: bridge.selectedIds.length > 1
            onTriggered: bridge.boolean("intersection")
        }
        Action {
            text: qsTr("&XOR (Exclusão Mútua)")
            enabled: bridge.selectedIds.length > 1
            onTriggered: bridge.boolean("xor")
        }
        Action {
            text: qsTr("&Dividir Regiões")
            enabled: bridge.selectedIds.length > 1
            onTriggered: bridge.boolean("divide")
        }

        MenuSeparator {}

        Action {
            text: qsTr("Ativar Ferramenta &Shape Builder")
            onTriggered: bridge.shapeBuilderStart()
        }
    }

    // -------------------------------------------------------------
    // Menu Janela
    // -------------------------------------------------------------
    Menu {
        title: qsTr("&Janela")

        Action {
            text: qsTr("Carregar Demonstração &Inioluwa Abiri")
            onTriggered: bridge.loadDemoDocument()
        }
    }

    // -------------------------------------------------------------
    // Menu Ajuda
    // -------------------------------------------------------------
    Menu {
        title: qsTr("A&juda")

        Action {
            text: qsTr("&Atalhos de Teclado...")
            shortcut: StandardKey.HelpContents
            onTriggered: desktopMenuBar.requestHelp()
        }
        Action {
            text: qsTr("&Sobre o Petunia Design Studio...")
            onTriggered: desktopMenuBar.requestHelp()
        }
    }
}
