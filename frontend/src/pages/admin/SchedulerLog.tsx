import { Link, useParams } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { schedulerApi } from '../../api/client';

const cardStyle = {
  backgroundColor: 'white',
  padding: '20px',
  borderRadius: '8px',
  boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
};

export default function SchedulerLog() {
  const { id } = useParams<{ id: string }>();
  const sid = id ? parseInt(id, 10) : NaN;

  const { data: item, isLoading: itemLoading } = useQuery({
    queryKey: ['scheduler', sid],
    queryFn: () => schedulerApi.getById(sid).then((r) => r.data),
    enabled: !Number.isNaN(sid),
  });

  const { data: logs, isLoading: logsLoading } = useQuery({
    queryKey: ['scheduler-log', sid],
    queryFn: () => schedulerApi.getLog(sid).then((r) => r.data),
    enabled: !Number.isNaN(sid),
  });

  if (Number.isNaN(sid)) return <div>Invalid scheduler ID</div>;
  if (itemLoading || !item) return <div>Loading...</div>;

  return (
    <div>
      <div style={{ marginBottom: '16px' }}>
        <Link to="/admin/scheduler" style={{ color: '#3498db' }}>← Back to Scheduler</Link>
      </div>
      <h1>Run history: {item.scheduler_name}</h1>
      <div style={cardStyle}>
        <p><strong>Subscription ID:</strong> {item.subscription_id}</p>
        <p><strong>Quantity:</strong> {item.quantity}</p>
        <p><strong>Start:</strong> {item.start_date ? new Date(item.start_date).toLocaleString() : '-'}</p>
        <p><strong>Next run:</strong> {item.next_run_time ? new Date(item.next_run_time).toLocaleString() : '-'}</p>
      </div>
      <div style={{ ...cardStyle, marginTop: '20px' }}>
        <h3>Metered audit logs</h3>
        {logsLoading ? (
          <p>Loading logs...</p>
        ) : (
          <div style={{ overflowX: 'auto', maxHeight: '50vh', overflowY: 'auto' }}>
            <table style={{ width: '100%', borderCollapse: 'collapse' }}>
              <thead style={{ backgroundColor: '#34495e', color: 'white' }}>
                <tr>
                  <th style={{ padding: '8px', textAlign: 'left' }}>ID</th>
                  <th style={{ padding: '8px', textAlign: 'left' }}>Created</th>
                  <th style={{ padding: '8px', textAlign: 'left' }}>Status</th>
                  <th style={{ padding: '8px', textAlign: 'left' }}>Request / Response</th>
                </tr>
              </thead>
              <tbody>
                {logs?.map((log) => (
                  <tr key={log.id} style={{ borderBottom: '1px solid #ecf0f1' }}>
                    <td style={{ padding: '8px' }}>{log.id}</td>
                    <td style={{ padding: '8px', whiteSpace: 'nowrap' }}>
                      {log.created_date ? new Date(log.created_date).toLocaleString() : '-'}
                    </td>
                    <td style={{ padding: '8px' }}>{log.status_code ?? '-'}</td>
                    <td style={{ padding: '8px', fontSize: '12px', wordBreak: 'break-all' }}>
                      {log.request_json ?? ''} / {log.response_json ?? ''}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
        {!logsLoading && (!logs || logs.length === 0) && (
          <p style={{ color: '#7f8c8d' }}>No run history yet.</p>
        )}
      </div>
    </div>
  );
}
