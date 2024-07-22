FROM python:3.9-slim

WORKDIR /app

COPY . /app

RUN pip install streamlit requests pandas streamlit-autorefresh plotly

CMD ["streamlit", "run", "app.py"]
