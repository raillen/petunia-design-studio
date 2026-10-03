//! Canonical EN and synchronized PT copy for desktop file workflows.
use crate::shell_strings::ShellString;
pub const FILE_WORKFLOW_STRINGS: &[ShellString] = &[
    ShellString { id: "ptnd.text.workflow.intent_perceptual", en: "Perceptual", pt: "Perceptivo" },
    ShellString { id: "ptnd.text.workflow.intent_relative", en: "Relative", pt: "Relativo" },
    ShellString { id: "ptnd.text.workflow.intent_saturation", en: "Saturation", pt: "Saturação" },
    ShellString { id: "ptnd.text.workflow.intent_absolute", en: "Absolute", pt: "Absoluto" },
    ShellString { id: "ptnd.text.workflow.bpc_on", en: "Black point compensation: on", pt: "Compensação de ponto preto: ligada" },
    ShellString { id: "ptnd.text.workflow.bpc_off", en: "Black point compensation: off", pt: "Compensação de ponto preto: desligada" },
    ShellString { id: "ptnd.text.workflow.icc_press", en: "Assign CMYK press ICC profile", pt: "Atribuir perfil ICC CMYK de impressão" },
    ShellString { id: "ptnd.text.workflow.icc_monitor", en: "Configure RGB monitor ICC profile", pt: "Configurar perfil ICC RGB do monitor" },
    ShellString { id: "ptnd.text.workflow.icc_assign", en: "Assign press profile…", pt: "Atribuir perfil de impressão…" },
    ShellString { id: "ptnd.text.workflow.icc_monitor_choose", en: "Monitor profile…", pt: "Perfil do monitor…" },
    ShellString { id: "ptnd.text.workflow.icc_missing", en: "Assign an ICC press profile before applying CMYK ink", pt: "Atribua um perfil ICC de impressão antes de aplicar tinta CMYK" },
    ShellString { id: "ptnd.text.workflow.icc_swatches", en: "ICC display preview", pt: "Prévia de exibição ICC" },
    ShellString { id: "ptnd.text.workflow.gamut_unmeasured", en: "Gamut has not been measured", pt: "Gama não medida" },
    ShellString { id: "ptnd.text.workflow.histogram_mean", en: "Mean", pt: "Média" },
    ShellString { id: "ptnd.text.workflow.histogram_shadows", en: "Shadows", pt: "Sombras" },
    ShellString { id: "ptnd.text.workflow.histogram_midtones", en: "Midtones", pt: "Meios" },
    ShellString { id: "ptnd.text.workflow.histogram_highlights", en: "Highlights", pt: "Realces" },
    ShellString { id: "ptnd.text.workflow.histogram_scope", en: "Visible composite · alpha-weighted · up to 512 px", pt: "Composição visível · ponderada pelo alfa · até 512 px" },
    ShellString { id: "ptnd.text.workflow.text_canvas_hint", en: "Edit text · Ctrl+Enter applies · Esc cancels · Shift selects", pt: "Editar texto · Ctrl+Enter aplica · Esc cancela · Shift seleciona" },
    ShellString { id: "ptnd.text.workflow.export_cancelled", en: "Export cancellation requested", pt: "Cancelamento da exportação solicitado" },
    ShellString { id: "ptnd.text.workflow.export_preview", en: "Active artboard preview · fitted to screen · checkerboard is transparency", pt: "Prévia da prancheta ativa · ajustada à tela · quadriculado indica transparência" },
    ShellString { id: "ptnd.text.workflow.recovery_available", en: "Recovery copies are available. Choose one to restore in a new unsaved tab; originals are preserved.", pt: "Há cópias de recuperação. Escolha uma para restaurar em uma nova aba não salva; os originais são preservados." },
    ShellString { id: "ptnd.text.workflow.recovery_continue", en: "Continue without restoring", pt: "Continuar sem restaurar" },
    ShellString { id: "ptnd.text.workflow.file_working", en: "Working on the file…", pt: "Processando o arquivo…" },
    ShellString { id: "ptnd.text.workflow.file_new_edits", en: "The copy was saved. Newer edits remain unsaved; the document stays open.", pt: "A cópia foi salva. Alterações mais recentes ainda não foram salvas; o documento permanece aberto." },
    ShellString { id: "ptnd.text.workflow.type_edit", en: "Typography…", pt: "Tipografia…" },
    ShellString { id: "ptnd.text.workflow.type_family", en: "Font family", pt: "Família da fonte" },
    ShellString { id: "ptnd.text.workflow.type_size", en: "Size (pt)", pt: "Tamanho (pt)" },
    ShellString { id: "ptnd.text.workflow.type_weight", en: "Weight (1–1000)", pt: "Peso (1–1000)" },
    ShellString { id: "ptnd.text.workflow.type_leading", en: "Line height (×)", pt: "Entrelinha (×)" },
    ShellString { id: "ptnd.text.workflow.type_tracking", en: "Tracking (pt)", pt: "Espaçamento (pt)" },
    ShellString { id: "ptnd.text.workflow.type_italic_on", en: "Italic: on", pt: "Itálico: ligado" },
    ShellString { id: "ptnd.text.workflow.type_italic_off", en: "Italic: off", pt: "Itálico: desligado" },
    ShellString { id: "ptnd.text.workflow.type_frame", en: "Flow: frame", pt: "Fluxo: quadro" },
    ShellString { id: "ptnd.text.workflow.type_artistic", en: "Flow: artistic", pt: "Fluxo: artístico" },
    ShellString { id: "ptnd.text.workflow.type_start", en: "Start", pt: "Início" },
    ShellString { id: "ptnd.text.workflow.type_center", en: "Center", pt: "Centro" },
    ShellString { id: "ptnd.text.workflow.type_end", en: "End", pt: "Fim" },
    ShellString { id: "ptnd.text.workflow.type_limits", en: "Frames wrap and clip; artistic text keeps explicit line breaks. Installed fallback fonts may be used.", pt: "Quadros quebram linhas e recortam o texto; texto artístico mantém as quebras explícitas. Fontes alternativas instaladas podem ser usadas." },
    ShellString { id: "ptnd.text.workflow.type_invalid", en: "Enter a valid font, finite metrics and a weight from 1 to 1000.", pt: "Informe uma fonte válida, medidas finitas e peso entre 1 e 1000." },
    ShellString { id: "ptnd.text.workflow.type_failed", en: "Typography could not be applied; the document was preserved.", pt: "Não foi possível aplicar a tipografia; o documento foi preservado." },
    ShellString { id: "ptnd.text.workflow.recover_title", en: "Restore recovery copy — Save As preserves the original", pt: "Restaurar cópia — Salvar como preserva o original" },
    ShellString { id: "ptnd.text.workflow.browse", en: "Browse…", pt: "Procurar…" },
    ShellString { id: "ptnd.text.workflow.cancel", en: "Cancel", pt: "Cancelar" },
    ShellString { id: "ptnd.text.workflow.close_discard", en: "Discard and close", pt: "Descartar e fechar" },
    ShellString { id: "ptnd.text.workflow.close_save", en: "Save and close", pt: "Salvar e fechar" },
    ShellString { id: "ptnd.text.workflow.close_title", en: "Unsaved changes", pt: "Alterações não salvas" },
    ShellString { id: "ptnd.text.workflow.close_one", en: "Save this document before closing?", pt: "Salvar este documento antes de fechar?" },
    ShellString { id: "ptnd.text.workflow.close_all", en: "Save all modified documents before closing?", pt: "Salvar todos os documentos alterados antes de fechar?" },
    ShellString { id: "ptnd.text.workflow.completed", en: "Completed", pt: "Concluído" },
    ShellString { id: "ptnd.text.workflow.destination", en: "Destination file", pt: "Arquivo de destino" },
    ShellString { id: "ptnd.text.workflow.failed", en: "Operation failed", pt: "A operação falhou" },
    ShellString { id: "ptnd.text.workflow.open", en: "Open document", pt: "Abrir documento" },
    ShellString { id: "ptnd.text.workflow.save", en: "Save document", pt: "Salvar documento" },
    ShellString { id: "ptnd.text.workflow.saved", en: "Saved", pt: "Salvo" },
    ShellString { id: "ptnd.text.workflow.path_hint", en: "Choose a file or enter its full path. Native document: .PTND.", pt: "Escolha um arquivo ou digite seu caminho completo. Documento nativo: .PTND." },
    ShellString { id: "ptnd.text.workflow.empty_path", en: "Choose a file first.", pt: "Escolha um arquivo primeiro." },
    ShellString { id: "ptnd.text.workflow.overwrite", en: "Replace existing file", pt: "Substituir arquivo existente" },
    ShellString { id: "ptnd.text.workflow.overwrite_reason", en: "A file already exists at this destination. Replace it?", pt: "Já existe um arquivo neste destino. Deseja substituí-lo?" },
    ShellString { id: "ptnd.text.workflow.picker_hint", en: "If the system dialog is unavailable or cancelled, enter the path below.", pt: "Se o diálogo do sistema estiver indisponível ou for cancelado, digite o caminho abaixo." },
    ShellString { id: "ptnd.text.workflow.export", en: "Export artwork", pt: "Exportar arte" },
    ShellString { id: "ptnd.text.workflow.format", en: "File format", pt: "Formato do arquivo" },
    ShellString { id: "ptnd.text.workflow.dpi", en: "PNG resolution (DPI)", pt: "Resolução do PNG (DPI)" },
    ShellString { id: "ptnd.text.workflow.scope_png", en: "Scope: active artboard", pt: "Escopo: prancheta ativa" },
    ShellString { id: "ptnd.text.workflow.scope_document", en: "Scope: whole document; PDF respects artboards enabled for export", pt: "Escopo: documento inteiro; o PDF respeita as pranchetas habilitadas para exportação" },
    ShellString { id: "ptnd.text.workflow.alpha", en: "PNG preserves transparency. White background is not available yet.", pt: "O PNG preserva a transparência. Fundo branco ainda não está disponível." },
    ShellString { id: "ptnd.text.workflow.pdf_limit", en: "Strict PDF subset: embedded fonts/images and assigned ICC ink; effects may require explicit rasterization. PDF/X-4 is unavailable.", pt: "PDF estrito: fontes/imagens incorporadas e tinta com ICC atribuído; efeitos podem exigir rasterização explícita. PDF/X-4 indisponível." },
    ShellString { id: "ptnd.text.workflow.color_unavailable", en: "Display P3 and CMYK document creation are unavailable; this document uses the current RGB workflow.", pt: "A criação de documentos Display P3 e CMYK está indisponível; este documento usa o fluxo RGB atual." },
    ShellString { id: "ptnd.text.workflow.exported", en: "Exported", pt: "Exportado" },
    ShellString { id: "ptnd.text.workflow.export_dimensions", en: "Output size", pt: "Tamanho de saída" },
    ShellString { id: "ptnd.text.workflow.new", en: "New document", pt: "Novo documento" },
    ShellString { id: "ptnd.text.workflow.name", en: "Document name", pt: "Nome do documento" },
    ShellString { id: "ptnd.text.workflow.presets", en: "Size presets", pt: "Predefinições de tamanho" },
    ShellString { id: "ptnd.text.workflow.square", en: "Square", pt: "Quadrado" },
    ShellString { id: "ptnd.text.workflow.mobile", en: "Mobile", pt: "Celular" },
    ShellString { id: "ptnd.text.workflow.dimensions", en: "Dimensions", pt: "Dimensões" },
    ShellString { id: "ptnd.text.workflow.rotate_orientation", en: "Swap orientation", pt: "Inverter orientação" },
    ShellString { id: "ptnd.text.workflow.bleed", en: "Bleed", pt: "Sangria" },
    ShellString { id: "ptnd.text.workflow.no_bleed", en: "No bleed", pt: "Sem sangria" },
    ShellString { id: "ptnd.text.workflow.margins", en: "Safe margins", pt: "Margens de segurança" },
    ShellString { id: "ptnd.text.workflow.color_mode", en: "Color mode", pt: "Modo de cor" },
    ShellString { id: "ptnd.text.workflow.create", en: "Create document", pt: "Criar documento" },
    ShellString { id: "ptnd.text.workflow.dismiss", en: "Dismiss message", pt: "Dispensar mensagem" },
    ShellString { id: "ptnd.text.workflow.edit_name", en: "Rename layer", pt: "Renomear camada" },
    ShellString { id: "ptnd.text.workflow.edit_name_hint", en: "Enter a name (1–256 characters).", pt: "Digite um nome (1–256 caracteres)." },
    ShellString { id: "ptnd.text.workflow.edit_text", en: "Edit text…", pt: "Editar texto…" },
    ShellString { id: "ptnd.text.workflow.edit_text_hint", en: "Edit content. Font, spacing and path attachment are preserved.", pt: "Edite o conteúdo. Fonte, espaçamento e vínculo ao caminho serão preservados." },
    ShellString { id: "ptnd.text.workflow.edit_transform", en: "Edit transform…", pt: "Editar transformação…" },
    ShellString { id: "ptnd.text.workflow.edit_frame_hint", en: "Local placement relative to the parent/artboard. Coordinates and dimensions: points; rotation: degrees.", pt: "Posição local em relação ao pai/prancheta. Coordenadas e dimensões: pontos; rotação: graus." },
    ShellString { id: "ptnd.text.workflow.edit_apply", en: "Apply", pt: "Aplicar" },
    ShellString { id: "ptnd.text.workflow.edit_applied", en: "Object updated", pt: "Objeto atualizado" },
    ShellString { id: "ptnd.text.workflow.edit_typography", en: "TYPOGRAPHY & TEXT", pt: "TIPOGRAFIA E TEXTO" },
    ShellString { id: "ptnd.text.workflow.edit_single_selection", en: "Precise transformation requires one selected object. Other inspector controls currently affect the first selected object.", pt: "A transformação precisa exige um objeto selecionado. Os demais controles atuais afetam o primeiro objeto da seleção." },
    ShellString { id: "ptnd.text.workflow.edit_target_changed", en: "The document or object is no longer the edit target. Cancel and reopen this editor.", pt: "O documento ou objeto já não corresponde ao alvo da edição. Cancele e reabra este editor." },
    ShellString { id: "ptnd.text.workflow.edit_property_changed", en: "This property changed while the editor was open. Your draft is preserved; cancel and reopen to use the current value.", pt: "Esta propriedade mudou enquanto o editor estava aberto. Seu rascunho foi preservado; cancele e reabra para usar o valor atual." },
    ShellString { id: "ptnd.text.workflow.edit_invalid_name", en: "Use a non-empty name of up to 256 characters, without control characters.", pt: "Use um nome não vazio de até 256 caracteres, sem caracteres de controle." },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Locale, LocalizationService};
    #[test]
    fn file_workflow_messages_resolve_in_both_release_locales() {
        let localization = LocalizationService::with_shell_catalog();
        let mut ids = std::collections::HashSet::new();
        for row in FILE_WORKFLOW_STRINGS {
            assert!(ids.insert(row.id), "duplicate workflow id");
            assert!(!crate::shell_strings::SHELL_STRINGS
                .iter()
                .any(|other| other.id == row.id));
            assert_eq!(localization.text(row.id, &Locale::EnUs), row.en);
            assert_eq!(localization.text(row.id, &Locale::PtBr), row.pt);
        }
    }
}
