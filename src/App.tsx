import { StickyNote } from './components/StickyNote';
import { ErrorBoundary } from './components/ErrorBoundary';

function App() {
  // Extract noteId from URL query param: ?noteId=xxx
  const params = new URLSearchParams(window.location.search);
  const noteId = params.get('noteId');

  if (!noteId) {
    // This is the hidden manager window — render nothing
    return null;
  }

  return (
    <ErrorBoundary>
      <StickyNote noteId={noteId} />
    </ErrorBoundary>
  );
}

export default App;
