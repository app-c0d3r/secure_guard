import React from 'react';
import { ChakraProvider, defaultSystem } from '@chakra-ui/react';
import SimpleApp from './SimpleApp';

const App: React.FC = () => {
  return (
    <ChakraProvider value={defaultSystem}>
      <SimpleApp />
    </ChakraProvider>
  );
};

export default App;
