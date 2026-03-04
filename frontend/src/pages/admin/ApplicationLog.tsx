import { useQuery } from '@tanstack/react-query';
import { applicationLogsApi } from '../../api/client';

const cardStyle = {
  backgroundColor: 'white',
  padding: '20px',
  borderRadius: '8px',
  boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
};

export default function ApplicationLog() {
  const { data: logs, isLoading } = useQuery({
    queryKey: ['application-logs'],
    queryFn: () => applicationLogsApi.getAll().then((r) => r.data),
  });

  if (isLoading) return <div>Loading...</div>;

  return (
    <div>
      <h1>Application Logs</h1>
      <div style={cardStyle}>
        <p style={{ color: '#7f8c8d', marginBottom: '16px' }}>
          Recent application activity (ordered by time descending).
        </p>
        <div style={{ overflowX: 'auto', maxHeight: '70vh', overflowY: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead style={{ position: 'sticky', top: 0, backgroundColor: '#34495e', color: 'white' }}>
              <tr>
                <th style={{ padding: '12px', textAlign: 'left' }}>ID</th>
                <th style={{ padding: '12px', textAlign: 'left' }}>Action Time</th>
                <th style={{ padding: '12px', textAlign: 'left' }}>Log Details</th>
              </tr>
            </thead>
            <tbody>
              {logs?.map((log) => (
                <tr key={log.id} style={{ borderBottom: '1px solid #ecf0f1' }}>
                  <td style={{ padding: '12px' }}>{log.id}</td>
                  <td style={{ padding: '12px', whiteSpace: 'nowrap' }}>
                    {log.action_time ? new Date(log.action_time).toLocaleString() : '-'}
                  </td>
                  <td style={{ padding: '12px', wordBreak: 'break-word' }}>{log.log_detail ?? '-'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        {(!logs || logs.length === 0) && (
          <p style={{ color: '#7f8c8d', marginTop: '20px' }}>No application logs found.</p>
        )}
      </div>
    </div>
  );
}
