import tempfile

import uvicorn
from fastapi import FastAPI, File, UploadFile

app = FastAPI()


@app.post("/upload_file/")
def upload(file: UploadFile = File(...)):
    try:
        with tempfile.TemporaryFile() as tf:
            while contents := file.file.read(1024 * 1024):
                tf.write(contents)
    except Exception:
        return {"message": "There was an error uploading the file"}
    finally:
        file.file.close()

    return {"message": f"Successfully uploaded file: {file.filename}"}


if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8181, log_level="debug")
