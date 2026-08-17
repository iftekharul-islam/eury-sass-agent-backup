import type { GeneratedImagePreview } from '../../lib/conversations';

export function ChatGeneratedImages({ images }: { images: GeneratedImagePreview[] }) {
  if (!images.length) return null;

  return (
    <div className="chat-generated-images">
      {images.map((image, index) => (
        <figure key={`${image.dataUrl.slice(0, 32)}-${index}`} className="chat-generated-image">
          <img src={image.dataUrl} alt={image.caption ?? 'Generated image'} />
          {image.caption ? <figcaption>{image.caption}</figcaption> : null}
        </figure>
      ))}
    </div>
  );
}
