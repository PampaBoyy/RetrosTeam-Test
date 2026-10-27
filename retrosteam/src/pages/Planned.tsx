export function Planned({ title, milestone, text }: { title: string; milestone: string; text: string }) {
  return (
    <>
      <h1>{title}</h1>
      <p className="sub"><span className="badge warn">Planejado — {milestone}</span></p>
      <div className="empty">{text} Nada nesta tela está funcional ainda.</div>
    </>
  );
}
