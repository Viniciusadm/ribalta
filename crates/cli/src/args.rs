use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Lang {
    Pt,
    En,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Self::Pt => "pt",
            Self::En => "en",
        }
    }

    pub fn locale(self) -> ribalta::Locale {
        ribalta::Locale::from_code(self.code())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Profile {
    #[value(help = "Sem restrições adicionais")]
    Default,
    #[value(help = "Aula em texto: sem imagens externas, sons, campos, @persist, next nem hidden")]
    Lesson,
}

impl Profile {
    pub fn compiler(self) -> ribalta::Profile {
        match self {
            Self::Default => ribalta::Profile::Default,
            Self::Lesson => ribalta::Profile::Lesson,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum LanguageCommand {
    #[command(about = "Compila um arquivo .ribalta localmente e mostra os diagnósticos")]
    Check(CheckArgs),

    #[command(about = "Formata arquivos .ribalta sem mudar o resultado compilado")]
    Fmt(FmtArgs),

    #[command(about = "Mostra o mapa de rotas entre páginas: escolhas, finais e páginas inalcançáveis")]
    Graph(GraphArgs),

    #[command(about = "Servidor de linguagem (LSP) para editores, pela entrada e saída padrão")]
    Lsp,
}

impl LanguageCommand {
    pub fn needs_asset_base(&self) -> bool {
        matches!(self, Self::Check(args) if args.check_assets)
    }
}

#[derive(Debug, Args)]
pub struct CheckArgs {
    #[arg(help = "Arquivo .ribalta, ou `-` para ler da entrada padrão")]
    pub file: PathBuf,

    #[arg(long, value_name = "ARQUIVO", help = "Tema base aplicado antes do documento")]
    pub theme: Option<PathBuf>,

    #[arg(long, value_name = "LARGURAxALTURA", value_parser = parse_page, help = "Tamanho de página padrão, como faria o host: 1920x1080 ou 800xauto (padrão: 1920xauto)")]
    pub page: Option<ribalta::PageSize>,

    #[arg(long, value_enum, default_value_t = Profile::Default, help = "Perfil de validação: `lesson` recusa imagens externas, sons, campos, @persist, next e hidden")]
    pub profile: Profile,

    #[arg(long, help = "Trata avisos como erros")]
    pub strict: bool,

    #[arg(long, help = "Imprime os diagnósticos em JSON")]
    pub json: bool,

    #[arg(long, help = "Confere pela internet se as imagens e os sons respondem (aviso R801)")]
    pub check_assets: bool,
}

#[derive(Debug, Args)]
pub struct FmtArgs {
    #[arg(help = "Arquivos ou pastas (padrão: a pasta atual), ou `-` para formatar a entrada padrão")]
    pub paths: Vec<PathBuf>,

    #[arg(long, help = "Só lista os arquivos que mudariam e falha se houver algum")]
    pub check: bool,

    #[arg(long, value_name = "ARQUIVO", help = "Tema base usado para conferir que o resultado não muda")]
    pub theme: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct GraphArgs {
    #[arg(help = "Arquivo .ribalta, ou `-` para ler da entrada padrão")]
    pub file: PathBuf,

    #[arg(long, value_name = "ARQUIVO", help = "Tema base aplicado antes do documento")]
    pub theme: Option<PathBuf>,

    #[arg(long, value_enum, default_value_t = GraphFormat::Text, help = "Formato da saída")]
    pub format: GraphFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum GraphFormat {
    #[value(help = "Lista legível de páginas e destinos")]
    Text,
    #[value(help = "JSON com páginas e arestas, para scripts")]
    Json,
    #[value(help = "Diagrama Mermaid (cole no GitHub, Notion ou mermaid.live)")]
    Mermaid,
    #[value(help = "Graphviz DOT (dot -Tsvg)")]
    Dot,
}

fn parse_page(text: &str) -> Result<ribalta::PageSize, String> {
    ribalta::parse_page_size(text).ok_or_else(|| format!("tamanho de página inválido: `{text}` (use 1920x1080 ou 800xauto)"))
}
