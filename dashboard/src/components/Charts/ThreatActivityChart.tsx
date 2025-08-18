import React, { useMemo } from 'react';
import {
  ResponsiveContainer,
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  AreaChart,
  Area,
} from 'recharts';
import { Box } from '@chakra-ui/react';

interface ThreatAlert {
  alert_id: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  created_at: string;
  status: string;
}

interface ThreatActivityChartProps {
  alerts: ThreatAlert[];
}

const ThreatActivityChart: React.FC<ThreatActivityChartProps> = ({ alerts }) => {
  const gridColor = '#f7fafc';
  const textColor = '#4a5568';

  const chartData = useMemo(() => {
    // Group alerts by hour for the last 24 hours
    const now = new Date();
    const data = [];

    for (let i = 23; i >= 0; i--) {
      const hour = new Date(now.getTime() - i * 60 * 60 * 1000);
      const hourStart = new Date(hour);
      hourStart.setMinutes(0, 0, 0);
      const hourEnd = new Date(hourStart.getTime() + 60 * 60 * 1000);

      const hourAlerts = alerts.filter(alert => {
        const alertTime = new Date(alert.created_at);
        return alertTime >= hourStart && alertTime < hourEnd;
      });

      const severityCounts = {
        critical: hourAlerts.filter(a => a.severity === 'critical').length,
        high: hourAlerts.filter(a => a.severity === 'high').length,
        medium: hourAlerts.filter(a => a.severity === 'medium').length,
        low: hourAlerts.filter(a => a.severity === 'low').length,
      };

      data.push({
        time: hourStart.getHours().toString().padStart(2, '0') + ':00',
        fullTime: hourStart.toISOString(),
        critical: severityCounts.critical,
        high: severityCounts.high,
        medium: severityCounts.medium,
        low: severityCounts.low,
        total: hourAlerts.length,
      });
    }

    return data;
  }, [alerts]);

  const CustomTooltip = ({ active, payload, label }: any) => {
    if (active && payload && payload.length) {
      return (
        <Box
          bg="white"
          p={3}
          borderRadius="md"
          shadow="lg"
          border="1px"
          borderColor="gray.200"
        >
          <Box fontWeight="semibold" mb={2}>
            {label}
          </Box>
          {payload.map((entry: any, index: number) => (
            <Box key={index} color={entry.color} fontSize="sm">
              {entry.name}: {entry.value}
            </Box>
          ))}
        </Box>
      );
    }
    return null;
  };

  return (
    <Box height="300px" width="100%">
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" stroke={gridColor} />
          <XAxis 
            dataKey="time" 
            stroke={textColor}
            fontSize={12}
            interval="preserveStartEnd"
          />
          <YAxis stroke={textColor} fontSize={12} />
          <Tooltip content={<CustomTooltip />} />
          <Legend />
          <Area
            type="monotone"
            dataKey="critical"
            stackId="1"
            stroke="#E53E3E"
            fill="#E53E3E"
            fillOpacity={0.8}
            name="Critical"
          />
          <Area
            type="monotone"
            dataKey="high"
            stackId="1"
            stroke="#FF8C00"
            fill="#FF8C00"
            fillOpacity={0.8}
            name="High"
          />
          <Area
            type="monotone"
            dataKey="medium"
            stackId="1"
            stroke="#F6AD55"
            fill="#F6AD55"
            fillOpacity={0.8}
            name="Medium"
          />
          <Area
            type="monotone"
            dataKey="low"
            stackId="1"
            stroke="#48BB78"
            fill="#48BB78"
            fillOpacity={0.8}
            name="Low"
          />
        </AreaChart>
      </ResponsiveContainer>
    </Box>
  );
};

export default ThreatActivityChart;