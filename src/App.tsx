import { StickyNote } from './components/StickyNote';

function App() {
  // Extract noteId from URL query param: ?noteId=xxx
  const params = new URLSearchParams(window.location.search);
  const noteId = params.get('noteId');

  if (!noteId) {
    // This is the hidden manager window — render nothing
    return null;
  }

  return <StickyNote noteId={noteId} />;
}

export default App;
