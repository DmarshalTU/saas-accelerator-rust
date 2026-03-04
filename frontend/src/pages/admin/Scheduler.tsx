import { Link } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { schedulerApi, subscriptionsApi, plansApi } from '../../api/client';
import { useState, useEffect, useMemo } from 'react';

const cardStyle = {
  backgroundColor: 'white',
  padding: '20px',
  borderRadius: '8px',
  boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
};

export default function Scheduler() {
  const queryClient = useQueryClient();
  const [showAdd, setShowAdd] = useState(false);
  const [form, setForm] = useState({
    scheduler_name: '',
    subscription_id: 0,
    plan_id: 0,
    dimension_id: 0,
    frequency_id: 0,
    quantity: 1,
    start_date: new Date().toISOString().slice(0, 16),
  });

  const { data: list, isLoading } = useQuery({
    queryKey: ['scheduler'],
    queryFn: () => schedulerApi.getList().then((r) => r.data),
  });

  const { data: subscriptions } = useQuery({
    queryKey: ['subscriptions'],
    queryFn: () => subscriptionsApi.getAll().then((r) => r.data),
  });

  const { data: frequencies } = useQuery({
    queryKey: ['scheduler-frequencies'],
    queryFn: () => schedulerApi.getFrequencies().then((r) => r.data),
  });

  const { data: dimensions } = useQuery({
    queryKey: ['scheduler-dimensions', form.subscription_id],
    queryFn: () => schedulerApi.getDimensionsBySubscription(form.subscription_id).then((r) => r.data),
    enabled: form.subscription_id > 0,
  });

  const { data: plans } = useQuery({
    queryKey: ['plans'],
    queryFn: () => plansApi.getAll().then((r) => r.data),
  });

  const activeSubscriptions = useMemo(
    () => subscriptions?.filter((s) => s.subscription_status === 'Subscribed' && s.is_active) ?? [],
    [subscriptions],
  );
  const selectedSubscription = useMemo(
    () => activeSubscriptions.find((s) => s.id === form.subscription_id),
    [activeSubscriptions, form.subscription_id],
  );

  useEffect(() => {
    if (!selectedSubscription || !plans?.length) return;
    const plan = plans.find((p) => p.plan_id === selectedSubscription.amp_plan_id);
    if (plan) setForm((f) => ({ ...f, plan_id: plan.id }));
  }, [selectedSubscription?.id, selectedSubscription?.amp_plan_id, plans]);

  const addMutation = useMutation({
    mutationFn: () =>
      schedulerApi.add({
        ...form,
        start_date: new Date(form.start_date).toISOString(),
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['scheduler'] });
      setShowAdd(false);
      setForm({ ...form, scheduler_name: '', quantity: 1, start_date: new Date().toISOString().slice(0, 16) });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: number) => schedulerApi.delete(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['scheduler'] }),
  });

  if (isLoading) return <div>Loading...</div>;

  return (
    <div>
      <h1>Scheduler (Metered plan triggers)</h1>
      <div style={cardStyle}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
          <p style={{ color: '#7f8c8d', margin: 0 }}>
            Schedule metered usage events. Ensure scheduler frequencies are enabled in Application Config.
          </p>
          <button
            type="button"
            onClick={() => setShowAdd(!showAdd)}
            style={{
              padding: '8px 16px',
              backgroundColor: '#27ae60',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            {showAdd ? 'Cancel' : 'Add new scheduled trigger'}
          </button>
        </div>

        {showAdd && (
          <div style={{ padding: '16px', backgroundColor: '#ecf0f1', borderRadius: '8px', marginBottom: '20px' }}>
            <h3>New scheduled trigger</h3>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px', maxWidth: '500px' }}>
              <label>
                Name
                <input
                  value={form.scheduler_name}
                  onChange={(e) => setForm((f) => ({ ...f, scheduler_name: e.target.value }))}
                  style={{ display: 'block', padding: '8px', width: '100%' }}
                />
              </label>
              <label>
                Subscription (internal id)
                <select
                  value={form.subscription_id || ''}
                  onChange={(e) => setForm((f) => ({ ...f, subscription_id: parseInt(e.target.value, 10) || 0 }))}
                  style={{ display: 'block', padding: '8px', width: '100%' }}
                >
                  <option value="">Select</option>
                  {activeSubscriptions.map((s) => (
                    <option key={s.id} value={s.id}>{s.amp_subscription_id} (id {s.id})</option>
                  ))}
                </select>
              </label>
              <label>
                Dimension
                <select
                  value={form.dimension_id || ''}
                  onChange={(e) => setForm((f) => ({ ...f, dimension_id: parseInt(e.target.value, 10) || 0 }))}
                  style={{ display: 'block', padding: '8px', width: '100%' }}
                >
                  <option value="">Select subscription first</option>
                  {dimensions?.map((d) => (
                    <option key={d.id} value={d.id}>{d.dimension}</option>
                  ))}
                </select>
              </label>
              <label>
                Frequency
                <select
                  value={form.frequency_id || ''}
                  onChange={(e) => setForm((f) => ({ ...f, frequency_id: parseInt(e.target.value, 10) || 0 }))}
                  style={{ display: 'block', padding: '8px', width: '100%' }}
                >
                  <option value="">Select</option>
                  {frequencies?.map((f) => (
                    <option key={f.id} value={f.id}>{f.frequency}</option>
                  ))}
                </select>
              </label>
              <label>
                Quantity
                <input
                  type="number"
                  min={0}
                  step="any"
                  value={form.quantity}
                  onChange={(e) => setForm((f) => ({ ...f, quantity: parseFloat(e.target.value) || 0 }))}
                  style={{ display: 'block', padding: '8px', width: '100%' }}
                />
              </label>
              <label>
                Start (local)
                <input
                  type="datetime-local"
                  value={form.start_date}
                  onChange={(e) => setForm((f) => ({ ...f, start_date: e.target.value }))}
                  style={{ display: 'block', padding: '8px', width: '100%' }}
                />
              </label>
            </div>
            <button
              type="button"
              onClick={() => addMutation.mutate()}
              disabled={!form.scheduler_name.trim() || !form.subscription_id || !form.dimension_id || !form.frequency_id || addMutation.isPending}
              style={{ marginTop: '12px', padding: '8px 16px', backgroundColor: '#3498db', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' }}
            >
              {addMutation.isPending ? 'Adding...' : 'Add'}
            </button>
          </div>
        )}

        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead style={{ backgroundColor: '#34495e', color: 'white' }}>
            <tr>
              <th style={{ padding: '12px', textAlign: 'left' }}>Id</th>
              <th style={{ padding: '12px', textAlign: 'left' }}>Name</th>
              <th style={{ padding: '12px', textAlign: 'left' }}>Sub ID</th>
              <th style={{ padding: '12px', textAlign: 'left' }}>Plan ID</th>
              <th style={{ padding: '12px', textAlign: 'left' }}>Dimension ID</th>
              <th style={{ padding: '12px', textAlign: 'left' }}>Quantity</th>
              <th style={{ padding: '12px', textAlign: 'left' }}>Start</th>
              <th style={{ padding: '12px', textAlign: 'left' }}>Next run</th>
              <th style={{ padding: '12px', textAlign: 'left' }}>Actions</th>
            </tr>
          </thead>
          <tbody>
            {list?.map((item) => (
              <tr key={item.id} style={{ borderBottom: '1px solid #ecf0f1' }}>
                <td style={{ padding: '12px' }}>{item.id}</td>
                <td style={{ padding: '12px' }}>{item.scheduler_name}</td>
                <td style={{ padding: '12px' }}>{item.subscription_id}</td>
                <td style={{ padding: '12px' }}>{item.plan_id}</td>
                <td style={{ padding: '12px' }}>{item.dimension_id}</td>
                <td style={{ padding: '12px' }}>{item.quantity}</td>
                <td style={{ padding: '12px' }}>{item.start_date ? new Date(item.start_date).toLocaleString() : '-'}</td>
                <td style={{ padding: '12px' }}>{item.next_run_time ? new Date(item.next_run_time).toLocaleString() : '-'}</td>
                <td style={{ padding: '12px' }}>
                  <Link to={`/admin/scheduler/${item.id}/log`} style={{ marginRight: '8px', color: '#3498db' }}>View log</Link>
                  <button
                    type="button"
                    onClick={() => { if (window.confirm('Delete this scheduler?')) deleteMutation.mutate(item.id); }}
                    disabled={deleteMutation.isPending}
                    style={{ padding: '6px 12px', backgroundColor: '#e74c3c', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' }}
                  >
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {(!list || list.length === 0) && !showAdd && (
          <p style={{ color: '#7f8c8d', marginTop: '20px' }}>No scheduled triggers. Add one to run metered usage on a schedule.</p>
        )}
      </div>
    </div>
  );
}
