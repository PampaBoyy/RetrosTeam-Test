export function Library() {
  return (
    <>
      <h1>Biblioteca</h1>
      <p className="sub">Seus jogos, sem mover seus arquivos.</p>
      <div className="row" style={{ marginBottom: 16 }}>
        <button className="btn" disabled title="Planejado para o Marco 2">Importar jogos</button>
        <span className="badge warn">Planejado — Marco 2</span>
      </div>
      <div className="empty">A biblioteca está vazia. A importação (referenciar, copiar ou mover com confirmação), busca, filtros e detalhes do jogo serão entregues no Marco 2. O banco já tem a tabela de jogos, com detecção de duplicatas por caminho.</div>
    </>
  );
}
