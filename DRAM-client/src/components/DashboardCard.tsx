type DashboardCardProps = {
  title: string;
  description: string;
  children: React.ReactNode;
};

function DashboardCard({ title, description, children }: DashboardCardProps) {
  return (
    <section className="dashboard-card">
      <div className="dashboard-card__header">
        <h3>{title}</h3>
        <p>{description}</p>
      </div>
      <div className="dashboard-card__content">{children}</div>
    </section>
  );
}

export default DashboardCard;