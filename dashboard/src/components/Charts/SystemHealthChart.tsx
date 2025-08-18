import React from 'react';
import {
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
  Tooltip,
  Legend,
} from 'recharts';
import { Box } from '@chakra-ui/react';

interface SystemHealthChartProps {
  cpuUsage: number;
  memoryUsage: number;
  diskUsage: number;
}

const SystemHealthChart: React.FC<SystemHealthChartProps> = ({
  cpuUsage,
  memoryUsage,
  diskUsage,
}) => {
  const textColor = '#4a5568';

  const data = [
    {
      name: 'CPU Usage',
      value: cpuUsage,
      color: '#4299E1',
    },
    {
      name: 'Memory Usage',
      value: memoryUsage,
      color: '#9F7AEA',
    },
    {
      name: 'Disk Usage',
      value: diskUsage,
      color: '#48BB78',
    },
  ];

  const CustomTooltip = ({ active, payload }: any) => {
    if (active && payload && payload.length) {
      const data = payload[0];
      return (
        <Box
          bg="white"
          p={3}
          borderRadius="md"
          shadow="lg"
          border="1px"
          borderColor="gray.200"
        >
          <Box fontWeight="semibold" color={data.payload.color}>
            {data.payload.name}: {data.value.toFixed(1)}%
          </Box>
        </Box>
      );
    }
    return null;
  };

  return (
    <Box height="250px" width="100%">
      <ResponsiveContainer width="100%" height="100%">
        <PieChart>
          <Pie
            data={data}
            cx="50%"
            cy="50%"
            outerRadius={80}
            innerRadius={40}
            paddingAngle={5}
            dataKey="value"
          >
            {data.map((entry, index) => (
              <Cell key={`cell-${index}`} fill={entry.color} />
            ))}
          </Pie>
          <Tooltip content={<CustomTooltip />} />
          <Legend />
        </PieChart>
      </ResponsiveContainer>
    </Box>
  );
};

export default SystemHealthChart;